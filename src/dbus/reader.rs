use crate::{
    dbus::{
        Message,
        decoders::{DecodingBuffer, HeaderDecoder, MessageDecoder},
    },
    liburing::IoUring,
    user_data::{ModuleId, UserData},
};
use anyhow::{Context as _, Result, ensure};

#[repr(u8)]
enum Op {
    ReadHeader,
    ReadBody,
}
const MAX_OP: u8 = Op::ReadBody as u8;

impl TryFrom<u8> for Op {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        ensure!(value <= MAX_OP);
        unsafe { Ok(std::mem::transmute::<u8, Self>(value)) }
    }
}

const HEADER_LEN: usize = HeaderDecoder::LENGTH + std::mem::size_of::<u32>();

#[derive(Debug)]
struct Buffer {
    bytes: Vec<u8>,
    target_len: usize,
    current_len: usize,
}

impl Buffer {
    fn new() -> Self {
        Self {
            bytes: vec![0; HEADER_LEN],
            target_len: HEADER_LEN,
            current_len: 0,
        }
    }

    fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.current_len]
    }

    fn set_body_len(&mut self, body_len: usize) {
        self.target_len += body_len;
        for _ in 0..body_len {
            self.bytes.push(0);
        }
    }

    fn remainder(&mut self) -> (&mut [u8], usize) {
        let blob = &mut self.bytes[self.current_len..];
        let len = self.target_len - self.current_len;
        (blob, len)
    }

    fn got_bytes(&mut self, len: usize) {
        self.current_len += len;
    }
}

pub(crate) struct Reader {
    fd: i32,
    module_id: ModuleId,
    buf: Buffer,
}

impl Reader {
    pub(crate) fn new(fd: i32, module_id: ModuleId) -> Self {
        Self {
            fd,
            module_id,
            buf: Buffer::new(),
        }
    }

    fn schedule_read_header(&mut self) -> Result<()> {
        let mut sqe = IoUring::get_sqe()?;
        let (bytes, len) = self.buf.remainder();
        sqe.prep_read(self.fd, bytes.as_mut_ptr(), len);
        sqe.set_user_data(UserData::new(self.module_id, Op::ReadHeader as u8));
        Ok(())
    }

    fn schedule_read_body(&mut self) -> Result<()> {
        let mut sqe = IoUring::get_sqe()?;
        let (bytes, len) = self.buf.remainder();
        sqe.prep_read(self.fd, bytes.as_mut_ptr(), len);
        sqe.set_user_data(UserData::new(self.module_id, Op::ReadBody as u8));
        Ok(())
    }

    pub(crate) fn init(&mut self) -> Result<()> {
        self.schedule_read_header()
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Result<Option<Message<'static>>> {
        match Op::try_from(op)? {
            Op::ReadHeader => {
                ensure!(res > 0, "res is {res}, buf is {:?}", self.buf);
                let bytes_read = res as usize;
                assert_eq!(bytes_read, HEADER_LEN);
                self.buf.got_bytes(bytes_read);
                let buf = self.buf.as_slice();

                let mut buf = DecodingBuffer::new(buf);
                let header = HeaderDecoder::decode(&mut buf)?;
                let header_fields_len = buf.peek_u32().context("EOF")? as usize;
                let body_len = header_fields_len.next_multiple_of(8) + header.body_len;
                self.buf.set_body_len(body_len);
                self.schedule_read_body()?;
                Ok(None)
            }
            Op::ReadBody => {
                ensure!(res >= 0);
                let bytes_read = res as usize;
                self.buf.got_bytes(bytes_read);

                if bytes_read == 0 {
                    let message = MessageDecoder::decode(self.buf.as_slice())?;
                    self.buf = Buffer::new();
                    self.schedule_read_header()?;
                    Ok(Some(message))
                } else {
                    self.schedule_read_body()?;
                    Ok(None)
                }
            }
        }
    }
}

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

pub(crate) struct Reader {
    fd: i32,
    module_id: ModuleId,
    buf: [u8; 5_000],
}

impl Reader {
    pub(crate) fn new(fd: i32, module_id: ModuleId) -> Self {
        Self {
            fd,
            module_id,
            buf: [0; _],
        }
    }

    fn schedule_read_header(&mut self, ring: &mut IoUring) -> Result<()> {
        let mut sqe = ring.get_sqe()?;
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), HEADER_LEN);
        sqe.set_user_data(UserData::new(self.module_id, Op::ReadHeader as u8));
        Ok(())
    }

    fn schedule_read_body(&mut self, len: usize, ring: &mut IoUring) -> Result<()> {
        let mut sqe = ring.get_sqe()?;
        sqe.prep_read(self.fd, self.buf[HEADER_LEN..].as_mut_ptr(), len);
        sqe.set_user_data(UserData::new(self.module_id, Op::ReadBody as u8));
        Ok(())
    }

    pub(crate) fn init(&mut self, ring: &mut IoUring) -> Result<()> {
        self.schedule_read_header(ring)
    }

    pub(crate) fn process(
        &mut self,
        op: u8,
        res: i32,
        ring: &mut IoUring,
    ) -> Result<Option<Message<'static>>> {
        match Op::try_from(op)? {
            Op::ReadHeader => {
                ensure!(res > 0, "res is {res}, buf is {:?}", &self.buf[..16]);
                let bytes_read = res as usize;
                assert_eq!(bytes_read, HEADER_LEN);
                let buf = &self.buf[..bytes_read];

                let mut buf = DecodingBuffer::new(buf);
                let header = HeaderDecoder::decode(&mut buf)?;
                let header_fields_len = buf.peek_u32().context("EOF")? as usize;
                let remaining_len = header_fields_len.next_multiple_of(8) + header.body_len;
                self.schedule_read_body(remaining_len, ring)?;
                Ok(None)
            }
            Op::ReadBody => {
                ensure!(res > 0);
                let bytes_read = res as usize;
                let buf = &self.buf[..HEADER_LEN + bytes_read];

                let message = MessageDecoder::decode(buf)?;
                self.schedule_read_header(ring)?;
                Ok(Some(message))
            }
        }
    }
}

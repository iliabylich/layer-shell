use crate::{
    dbus::{
        ConnectionKind, Message,
        decoders::{DecodingBuffer, HeaderDecoder, MessageDecoder},
    },
    liburing::IoUring,
    macros::report_and_exit,
    user_data::{ModuleId, UserData},
};

#[repr(u8)]
#[derive(Debug)]
enum Op {
    ReadHeader,
    ReadBody,
}
const MAX_OP: u8 = Op::ReadBody as u8;

impl From<u8> for Op {
    fn from(value: u8) -> Self {
        if value > MAX_OP {
            report_and_exit!("unsupported op in DBus Reader: {value}");
        }
        unsafe { std::mem::transmute::<u8, Self>(value) }
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
    healthy: bool,
}

impl Reader {
    pub(crate) fn new(kind: ConnectionKind) -> Self {
        let module_id = match kind {
            ConnectionKind::Session => ModuleId::SessionDBusReader,
            ConnectionKind::System => ModuleId::SystemDBusReader,
        };

        Self {
            fd: -1,
            module_id,
            buf: Buffer::new(),
            healthy: true,
        }
    }

    fn schedule_read_header(&mut self) {
        let mut sqe = IoUring::get_sqe();
        let (bytes, len) = self.buf.remainder();
        sqe.prep_read(self.fd, bytes.as_mut_ptr(), len);
        sqe.set_user_data(UserData::new(self.module_id, Op::ReadHeader as u8));
    }

    fn schedule_read_body(&mut self) {
        let mut sqe = IoUring::get_sqe();
        let (bytes, len) = self.buf.remainder();
        sqe.prep_read(self.fd, bytes.as_mut_ptr(), len);
        sqe.set_user_data(UserData::new(self.module_id, Op::ReadBody as u8));
    }

    pub(crate) fn init(&mut self, fd: i32) {
        self.fd = fd;
        self.schedule_read_header()
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Option<Message<'static>> {
        if !self.healthy {
            return None;
        }

        let op = Op::from(op);

        macro_rules! crash {
            ($($arg:tt)*) => {{
                log::error!($($arg)*);
                self.healthy = false;
                return None;
            }};
        }

        match op {
            Op::ReadHeader => {
                if res <= 0 {
                    crash!("{op:?}: res is {res}, buf is {:?}", self.buf);
                }
                let bytes_read = res as usize;
                assert_eq!(bytes_read, HEADER_LEN);
                self.buf.got_bytes(bytes_read);
                let buf = self.buf.as_slice();

                let mut buf = DecodingBuffer::new(buf);
                let header = match HeaderDecoder::decode(&mut buf) {
                    Ok(ok) => ok,
                    Err(err) => {
                        crash!("DBus Reader header decoding error: {err:?}");
                    }
                };
                let header_fields_len = match buf.peek_u32() {
                    Some(n) => n as usize,
                    None => {
                        crash!("Failed to read u32 from DBus message header");
                    }
                };
                let body_len = header_fields_len.next_multiple_of(8) + header.body_len;
                self.buf.set_body_len(body_len);
                self.schedule_read_body();
                None
            }
            Op::ReadBody => {
                if res < 0 {
                    crash!("{op:?}: res is {res}")
                }
                let bytes_read = res as usize;
                self.buf.got_bytes(bytes_read);

                if bytes_read == 0 {
                    let message = match MessageDecoder::decode(self.buf.as_slice()) {
                        Ok(ok) => ok,
                        Err(err) => {
                            crash!("{op:?}: failed to decode full DBus message: {err:?}");
                        }
                    };
                    self.buf = Buffer::new();
                    self.schedule_read_header();
                    Some(message)
                } else {
                    self.schedule_read_body();
                    None
                }
            }
        }
    }
}

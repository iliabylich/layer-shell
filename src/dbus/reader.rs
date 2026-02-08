use crate::{
    dbus::{
        ConnectionKind, Message,
        decoders::{DecodingBuffer, HeaderDecoder, MessageDecoder},
    },
    liburing::IoUring,
    macros::define_op,
    user_data::{ModuleId, UserData},
};

define_op!("DBus Reader", ReadHeader, ReadBody);

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
    kind: ConnectionKind,
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
            kind,
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
        sqe.set_user_data(UserData::new(self.module_id, Op::ReadHeader));
    }

    fn schedule_read_body(&mut self) {
        let mut sqe = IoUring::get_sqe();
        let (bytes, len) = self.buf.remainder();
        sqe.prep_read(self.fd, bytes.as_mut_ptr(), len);
        sqe.set_user_data(UserData::new(self.module_id, Op::ReadBody));
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

        macro_rules! assert_or_unhealthy {
            ($cond:expr, $($arg:tt)*) => {
                if !$cond {
                    log::error!("DBusReader({:?})::{op:?}", self.kind);
                    log::error!($($arg)*);
                    self.healthy = false;
                    return None;
                }
            };
        }

        match op {
            Op::ReadHeader => {
                assert_or_unhealthy!(res > 0, "res is {res}");
                let bytes_read = res as usize;
                assert_eq!(bytes_read, HEADER_LEN);
                self.buf.got_bytes(bytes_read);
                let buf = self.buf.as_slice();

                let mut buf = DecodingBuffer::new(buf);

                let header = HeaderDecoder::decode(&mut buf);
                assert_or_unhealthy!(header.is_ok(), "header decoding error: {header:?}");
                let header = unsafe { header.unwrap_unchecked() };

                let header_fields_len = buf.peek_u32();
                assert_or_unhealthy!(header_fields_len.is_some(), "failed to read u32");
                let header_fields_len = unsafe { header_fields_len.unwrap_unchecked() } as usize;

                let body_len = header_fields_len.next_multiple_of(8) + header.body_len;
                self.buf.set_body_len(body_len);
                self.schedule_read_body();
                None
            }
            Op::ReadBody => {
                assert_or_unhealthy!(res >= 0, "res is {res}");
                let bytes_read = res as usize;
                self.buf.got_bytes(bytes_read);

                if bytes_read == 0 {
                    let message = MessageDecoder::decode(self.buf.as_slice());
                    assert_or_unhealthy!(message.is_ok(), "mesage decoding error: {message:?}");
                    let message = unsafe { message.unwrap_unchecked() };

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

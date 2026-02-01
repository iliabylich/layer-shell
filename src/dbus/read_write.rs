use crate::{
    dbus::{
        Message,
        decoders::{DecodingBuffer, HeaderDecoder, MessageDecoder},
        encoders::MessageEncoder,
        serial::Serial,
    },
    liburing::IoUring,
    user_data::{ModuleId, UserData},
};
use anyhow::{Context as _, Result, ensure};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
enum ReadState {
    CanReadHeader,
    ReadingHeader,
    CanReadBody { remaining_len: usize },
    ReadingBody,
}

#[derive(Debug, Clone, Copy)]
enum WriteState {
    CanWrite,
    Writing,
}

pub(crate) struct ReadWrite {
    fd: i32,
    read_state: ReadState,
    read_buf: [u8; 5_000],
    write_state: WriteState,
    write_buf: Vec<u8>,
    queue: VecDeque<Vec<u8>>,
    serial: Serial,
    module_id: ModuleId,
}

#[repr(u8)]
enum Op {
    ReadHeader,
    ReadBody,
    Write,
}

const HEADER_LEN: usize = HeaderDecoder::LENGTH + std::mem::size_of::<u32>();

impl ReadWrite {
    pub(crate) fn new(
        fd: i32,
        queue: VecDeque<Vec<u8>>,
        serial: Serial,
        module_id: ModuleId,
    ) -> Self {
        Self {
            fd,
            read_state: ReadState::CanReadHeader,
            read_buf: [0; 5_000],
            write_state: WriteState::CanWrite,
            write_buf: vec![],
            queue,
            serial,
            module_id,
        }
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) {
        *message.serial_mut() = self.serial.increment_and_get();
        let bytes = MessageEncoder::encode(message);
        self.queue.push_back(bytes);
    }

    pub(crate) fn drain(&mut self, ring: &mut IoUring) -> Result<bool> {
        let mut drained = false;

        match self.write_state {
            WriteState::CanWrite => {
                if let Some(next) = self.queue.pop_front() {
                    self.write_buf = next;

                    let mut sqe = ring.get_sqe()?;
                    sqe.prep_write(self.fd, self.write_buf.as_ptr(), self.write_buf.len());
                    sqe.set_user_data(UserData::new(self.module_id, Op::Write as u8));

                    self.write_state = WriteState::Writing;
                    drained |= true;
                }
            }
            WriteState::Writing => {}
        }

        match self.read_state {
            ReadState::CanReadHeader => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_read(self.fd, self.read_buf.as_mut_ptr(), HEADER_LEN);
                sqe.set_user_data(UserData::new(self.module_id, Op::ReadHeader as u8));

                self.read_state = ReadState::ReadingHeader;
                drained |= true;
            }
            ReadState::ReadingHeader => {}

            ReadState::CanReadBody { remaining_len } => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_read(
                    self.fd,
                    self.read_buf[HEADER_LEN..].as_mut_ptr(),
                    remaining_len,
                );
                sqe.set_user_data(UserData::new(self.module_id, Op::ReadBody as u8));

                self.read_state = ReadState::ReadingBody;
                drained |= true;
            }
            ReadState::ReadingBody => {}
        }

        Ok(drained)
    }

    pub(crate) fn feed(&mut self, op_id: u8, res: i32) -> Result<Option<Message<'static>>> {
        if op_id == Op::Write as u8 {
            ensure!(
                matches!(self.write_state, WriteState::Writing),
                "malformed state, expected Writing, got {:?}",
                self.write_state
            );

            assert!(res > 0);
            let written = res as usize;
            assert_eq!(written, self.write_buf.len());

            self.write_buf.clear();
            self.write_state = WriteState::CanWrite;
            return Ok(None);
        }

        if op_id == Op::ReadHeader as u8 {
            ensure!(
                matches!(self.read_state, ReadState::ReadingHeader),
                "malformed state, expected ReadingHeader, got {:?}",
                self.read_state
            );

            assert!(res > 0, "res is {res}, buf is {:?}", &self.read_buf[..16]);
            let bytes_read = res as usize;
            assert_eq!(bytes_read, HEADER_LEN);
            let buf = &self.read_buf[..bytes_read];

            let mut buf = DecodingBuffer::new(buf);
            let header = HeaderDecoder::decode(&mut buf)?;
            let header_fields_len = buf.peek_u32().context("EOF")? as usize;
            let remaining_len = header_fields_len.next_multiple_of(8) + header.body_len;

            self.read_state = ReadState::CanReadBody { remaining_len };
            return Ok(None);
        }

        if op_id == Op::ReadBody as u8 {
            ensure!(
                matches!(self.read_state, ReadState::ReadingBody),
                "malformed state, expected ReadingBody, got {:?}",
                self.read_state
            );

            assert!(res > 0);
            let bytes_read = res as usize;
            let buf = &self.read_buf[..HEADER_LEN + bytes_read];

            let message = MessageDecoder::decode(buf)?;

            self.read_state = ReadState::CanReadHeader;
            return Ok(Some(message));
        }

        Ok(None)
    }
}

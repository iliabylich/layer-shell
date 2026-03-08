use crate::sansio::{Satisfy, Wants};
use anyhow::{Context, Result, ensure};
use std::collections::VecDeque;

pub(crate) struct DBusWriter {
    fd: i32,
    queue: VecDeque<Vec<u8>>,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    CanWrite,
    Waiting,
}

impl DBusWriter {
    pub(crate) fn new(fd: i32, queue: VecDeque<Vec<u8>>) -> Self {
        Self {
            fd,
            queue,
            state: State::CanWrite,
        }
    }

    pub(crate) fn enqueue(&mut self, message: Vec<u8>) {
        self.queue.push_back(message);
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match self.state {
            State::CanWrite => {
                if let Some(buf) = self.queue.front() {
                    self.state = State::Waiting;
                    Wants::Write {
                        fd: self.fd,
                        buf: buf.as_ptr(),
                        len: buf.len(),
                    }
                } else {
                    Wants::Nothing
                }
            }
            State::Waiting => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        ensure!(satisfy == Satisfy::Write);
        let message = self.queue.front().context(
            "malformed DBusWriter state: received Write, but there's no current message",
        )?;
        ensure!(res >= 0);
        let bytes_written = res as usize;
        ensure!(
            bytes_written == message.len(),
            "written is wrong: {bytes_written} vs {}",
            message.len()
        );

        let _ = self
            .queue
            .pop_front()
            .context("malformed DBusWriter state")?;
        self.state = State::CanWrite;
        Ok(())
    }
}

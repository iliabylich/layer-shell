use crate::sansio::{Satisfy, Wants, dbus::DBusQueue};
use anyhow::{Result, bail, ensure};

pub(crate) struct DBusWriter {
    fd: i32,
    current: Option<Vec<u8>>,
    queue: DBusQueue,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    CanWrite,
    Waiting,
}

impl DBusWriter {
    pub(crate) fn new(fd: i32, queue: DBusQueue) -> Self {
        let current = queue.pop_front();

        Self {
            fd,
            current,
            queue,
            state: State::CanWrite,
        }
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match self.state {
            State::CanWrite => {
                if self.current.is_none() {
                    self.current = self.queue.pop_front();
                }

                let Some(buf) = self.current.as_mut() else {
                    return Wants::Nothing;
                };

                self.state = State::Waiting;
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }
            State::Waiting => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        ensure!(satisfy == Satisfy::Write);
        ensure!(res >= 0);
        let Some(message) = self.current.take() else {
            bail!("malformed DBusWriter state: received Write, but there's no current message");
        };
        let bytes_written = res as usize;
        ensure!(
            bytes_written == message.len(),
            "written is wrong: {bytes_written} vs {}",
            message.len()
        );

        if let Some(next) = self.queue.pop_front() {
            self.current = Some(next);
        }
        self.state = State::CanWrite;
        Ok(())
    }
}

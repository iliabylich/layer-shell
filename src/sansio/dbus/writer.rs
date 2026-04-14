use crate::sansio::{DBusConnectionKind, Satisfy, Wants, dbus::DBusQueue};
use anyhow::{Result, bail, ensure};

pub(crate) struct DBusWriter {
    fd: i32,
    current: Option<Vec<u8>>,
    kind: DBusConnectionKind,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    CanWrite,
    WaitingForWrite,
    Dead,
}

impl DBusWriter {
    pub(crate) fn new(fd: i32, kind: DBusConnectionKind) -> Self {
        let current = DBusQueue::pop_front(kind);

        Self {
            fd,
            current,
            kind,
            state: State::CanWrite,
        }
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match self.state {
            State::CanWrite => {
                if self.current.is_none() {
                    self.current = DBusQueue::pop_front(self.kind);
                }

                let Some(buf) = self.current.as_mut() else {
                    return Wants::Nothing;
                };

                self.state = State::WaitingForWrite;
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }
            State::WaitingForWrite => Wants::Nothing,
            State::Dead => Wants::Nothing,
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        match (self.state, satisfy) {
            (State::Dead, _) => Ok(()),

            (State::WaitingForWrite, Satisfy::Write) => {
                ensure!(res >= 0, "DBusWriter::Write failed: {res}");
                let Some(message) = self.current.take() else {
                    bail!(
                        "malformed DBusWriter state: received Write, but there's no current message"
                    );
                };
                let bytes_written = res as usize;
                ensure!(
                    bytes_written == message.len(),
                    "written is wrong: {bytes_written} vs {}",
                    message.len()
                );

                if let Some(next) = DBusQueue::pop_front(self.kind) {
                    self.current = Some(next);
                }
                self.state = State::CanWrite;
                Ok(())
            }

            (state, satisfy) => {
                bail!("malformed DBusWriter state: {state:?} vs {satisfy:?}")
            }
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) {
        if let Err(err) = self.try_satisfy(satisfy, res) {
            log::error!("Module DBusWriter has crashed: {err:?}");
            self.stop();
        }
    }

    pub(crate) fn stop(&mut self) {
        log::error!("Stopping DBusWriter({:?})", self.kind);
        self.state = State::Dead;
    }
}

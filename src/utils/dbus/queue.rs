use dbus::{
    DBusSerial, EncodeError, OutgoingQueue, messages::org_freedesktop_dbus::Hello,
    messaging::DBusEncode,
};
use std::collections::VecDeque;

pub(crate) struct DBusQueue {
    serial: DBusSerial,
    q: VecDeque<Vec<u8>>,
}

impl OutgoingQueue for DBusQueue {
    fn push_raw_buf(&mut self, message: &[u8]) -> u32 {
        let serial = self.next_serial();
        let mut message = message.to_vec();
        if let Err(err) = DBusSerial::write_to_message(&mut message, serial) {
            unreachable!("buffer is too short: {err}");
        }
        self.q.push_back(message.clone());
        serial
    }

    fn peek(&self) -> Option<&[u8]> {
        self.q.front().map(Vec::as_slice)
    }

    fn pop(&mut self) {
        self.q.pop_front();
    }
}

impl DBusQueue {
    pub(crate) const fn new() -> Self {
        Self {
            serial: DBusSerial::new(),
            q: VecDeque::new(),
        }
    }

    fn next_serial(&mut self) -> u32 {
        let serial = self.serial.current();
        self.serial.advance();
        serial
    }

    pub(crate) fn push_hello(&mut self) -> Result<(), EncodeError> {
        self.push_and_discard_reply::<Hello>(())?;
        Ok(())
    }

    pub(crate) fn push_raw_buf(&mut self, buf: &[u8]) -> u32 {
        OutgoingQueue::push_raw_buf(self, buf)
    }

    pub(crate) fn push_and_discard_reply<M>(&mut self, args: M::Args<'_>) -> Result<(), EncodeError>
    where
        M: DBusEncode,
    {
        OutgoingQueue::push_and_discard_reply::<1_024, M>(self, args)
    }
}

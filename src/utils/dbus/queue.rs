use alloc::{collections::VecDeque, vec::Vec};
use dbus::{
    DBusSerial, EncodeError, OutgoingQueue, messages::org_freedesktop_dbus::Hello,
    messaging::DBusEncode,
};

pub(crate) struct SessionDBusQueue {
    serial: DBusSerial,
    q: VecDeque<Vec<u8>>,
}

impl OutgoingQueue for SessionDBusQueue {
    fn push_raw(&mut self, message: &[u8]) -> u32 {
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

impl SessionDBusQueue {
    pub(crate) fn new() -> Result<Self, EncodeError> {
        let mut this = Self {
            serial: DBusSerial::new(),
            q: VecDeque::new(),
        };

        let mut buf = [0; 1_024];
        let buf = Hello::encode((), &mut buf)?;
        this.push_raw(buf);

        Ok(this)
    }

    fn next_serial(&mut self) -> u32 {
        let serial = self.serial.current();
        self.serial.advance();
        serial
    }

    pub(crate) fn push_raw(&mut self, buf: &[u8]) -> u32 {
        OutgoingQueue::push_raw(self, buf)
    }
}

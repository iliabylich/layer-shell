use dbus::{
    DBusSerial, EncodeError, OutgoingQueue, messages::org_freedesktop_dbus::Hello,
    messaging::DBusEncode,
};
use std::collections::VecDeque;

macro_rules! generate_dbus_queue_impl {
    ($name:ident) => {
        pub(crate) struct $name {
            serial: DBusSerial,
            q: VecDeque<Vec<u8>>,
        }

        impl OutgoingQueue for $name {
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

        impl $name {
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
                let mut buf = [0; 1_024];
                let buf = Hello::encode((), &mut buf)?;
                self.push_raw(buf);
                Ok(())
            }

            pub(crate) fn push_raw(&mut self, buf: &[u8]) -> u32 {
                OutgoingQueue::push_raw(self, buf)
            }
        }
    };
}

generate_dbus_queue_impl!(SessionDBusQueue);
generate_dbus_queue_impl!(SystemDBusQueue);

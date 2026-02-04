use crate::{
    dbus::{DBus, Message, messages::message_is, types::Value},
    liburing::IoUring,
};
use anyhow::Result;

pub(crate) trait OneshotResource {
    type Input;
    type Output;

    fn make_request(&self, input: Self::Input) -> Message<'static>;
    fn try_process(&self, body: &[Value]) -> Result<Self::Output>;
}

#[derive(Debug, Clone, Copy)]
enum OneshotState {
    None,
    WaitingForReply(u32),
    ReplyReceived,
}

pub(crate) struct Oneshot<T>
where
    T: OneshotResource,
{
    resource: T,
    state: OneshotState,
}

impl<T> Oneshot<T>
where
    T: OneshotResource,
{
    pub(crate) fn new(resource: T) -> Self {
        Self {
            state: OneshotState::None,
            resource,
        }
    }

    pub(crate) fn start(
        &mut self,
        dbus: &mut DBus,
        input: T::Input,
        ring: &mut IoUring,
    ) -> Result<()> {
        if !matches!(self.state, OneshotState::None) {
            return Ok(());
        };

        let mut message = self.resource.make_request(input);
        dbus.enqueue(&mut message, ring)?;
        let reply_serial = message.serial();
        self.state = OneshotState::WaitingForReply(reply_serial);
        Ok(())
    }

    fn try_process(&self, message: &Message) -> Result<T::Output> {
        message_is!(message, Message::MethodReturn { body, .. });
        self.resource.try_process(body)
    }

    pub(crate) fn process(&mut self, message: &Message) -> Option<T::Output> {
        let OneshotState::WaitingForReply(reply_serial) = self.state else {
            return None;
        };
        if message.reply_serial() != Some(reply_serial) {
            return None;
        }
        self.state = OneshotState::ReplyReceived;

        self.try_process(message).ok()
    }

    pub(crate) fn reset(&mut self) {
        self.state = OneshotState::None;
    }
}

impl<T> std::fmt::Debug for Oneshot<T>
where
    T: OneshotResource,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OneshotHandler")
            .field("state", &self.state)
            .finish()
    }
}

use crate::{
    dbus::{Message, types::Value},
    sansio::DBusQueue,
};
use anyhow::{Result, bail};

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
    queue: DBusQueue,
}

impl<T> Oneshot<T>
where
    T: OneshotResource,
{
    pub(crate) fn new(resource: T, queue: DBusQueue) -> Self {
        Self {
            state: OneshotState::None,
            resource,
            queue,
        }
    }

    pub(crate) fn start(&mut self, input: T::Input) {
        if !matches!(self.state, OneshotState::None) {
            return;
        };

        let mut message = self.resource.make_request(input);
        self.queue.push_back(&mut message);
        let reply_serial = message.serial();
        self.state = OneshotState::WaitingForReply(reply_serial);
    }

    fn try_process(&self, message: &Message) -> Result<Option<T::Output>> {
        match message {
            Message::Error { error_name, .. } => {
                bail!("DBus error: {error_name}")
            }
            Message::MethodReturn { body, .. } => Ok(self.resource.try_process(body).ok()),
            _ => Ok(None),
        }
    }

    pub(crate) fn process(&mut self, message: &Message) -> Result<Option<T::Output>> {
        let OneshotState::WaitingForReply(reply_serial) = self.state else {
            return Ok(None);
        };
        if message.reply_serial() != Some(reply_serial) {
            return Ok(None);
        }
        self.state = OneshotState::ReplyReceived;

        self.try_process(message)
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

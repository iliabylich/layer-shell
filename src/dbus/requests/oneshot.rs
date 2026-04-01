use crate::{
    dbus::{
        OutgoingMessage,
        decoder::{Body, IncomingMessage, MessageType},
    },
    sansio::DBusQueue,
};
use anyhow::{Result, bail};

pub(crate) trait OneshotResource {
    type Input;
    type Output;

    fn request(&self, input: Self::Input) -> impl Into<OutgoingMessage>;
    fn try_recv(&self, body: Body<'_>) -> Result<Self::Output>;
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

    pub(crate) fn send(&mut self, input: T::Input) {
        if !matches!(self.state, OneshotState::None) {
            return;
        };

        let message = self.resource.request(input).into();
        let reply_serial = self.queue.push_back(message);
        self.state = OneshotState::WaitingForReply(reply_serial);
    }

    pub(crate) fn try_rev(&mut self, message: IncomingMessage<'_>) -> Result<Option<T::Output>> {
        let OneshotState::WaitingForReply(reply_serial) = self.state else {
            return Ok(None);
        };
        if message.reply_serial != Some(reply_serial) {
            return Ok(None);
        }
        self.state = OneshotState::ReplyReceived;

        match message.message_type {
            MessageType::Error => {
                bail!("DBus error: {:?}", message.error_name)
            }
            MessageType::MethodReturn => {
                if let Some(body) = message.body {
                    Ok(self.resource.try_recv(body).ok())
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.state = OneshotState::None;
    }
}

impl<T> core::fmt::Debug for Oneshot<T>
where
    T: OneshotResource,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OneshotHandler")
            .field("state", &self.state)
            .finish()
    }
}

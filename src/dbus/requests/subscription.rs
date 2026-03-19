use crate::{
    dbus::{
        OutgoingMessage,
        decoder::{Body, IncomingMessage, MessageType},
        messages::{
            interface_is,
            org_freedesktop_dbus::{AddMatch, RemoveMatch},
        },
    },
    ffi::ShortString,
    sansio::DBusQueue,
};
use anyhow::{Context as _, Result, ensure};

pub(crate) trait SubscriptionResource {
    type Output: std::fmt::Debug;

    fn set_path(&mut self, path: ShortString);
    fn try_process(&self, path: ShortString, body: Body<'_>) -> Result<Self::Output>;
}

pub(crate) struct Subscription<S>
where
    S: SubscriptionResource,
{
    path: Option<ShortString>,
    resource: S,
    queue: DBusQueue,
}

impl<S> Subscription<S>
where
    S: SubscriptionResource,
{
    pub(crate) fn new(resource: S, queue: DBusQueue) -> Self {
        Self {
            path: None,
            resource,
            queue,
        }
    }

    fn unsubscribe(&mut self) {
        let Some(old_path) = self.path.take() else {
            return;
        };

        let mut message: OutgoingMessage = RemoveMatch::new(old_path).into();
        self.queue.push_back(&mut message);
    }

    fn subscribe(&mut self, sender: ShortString, path: ShortString) {
        let mut message: OutgoingMessage = AddMatch::new(sender, path).into();
        self.queue.push_back(&mut message);
        self.path = Some(path);
        self.resource.set_path(path);
    }

    pub(crate) fn start(&mut self, sender: ShortString, path: ShortString) {
        self.unsubscribe();
        self.subscribe(sender, path);
    }

    pub(crate) fn reset(&mut self) {
        self.unsubscribe()
    }

    fn try_process(&self, message: IncomingMessage<'_>) -> Result<S::Output> {
        ensure!(message.message_type == MessageType::Signal);

        let interface = message.interface.context("no Interface")?;
        interface_is!(interface, "org.freedesktop.DBus.Properties");
        let path = message.path.context("no Path")?;
        let body = message.body.context("no Body")?;

        self.resource.try_process(ShortString::from(path), body)
    }

    pub(crate) fn process(&self, message: IncomingMessage<'_>) -> Option<S::Output> {
        self.try_process(message).ok()
    }
}

impl<S> std::fmt::Debug for Subscription<S>
where
    S: SubscriptionResource,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription")
            .field("path", &self.path)
            .finish()
    }
}

use crate::{
    dbus::{
        Message,
        decoder::{Body, IncomingMessage, MessageType},
        messages::org_freedesktop_dbus::{AddMatch, RemoveMatch},
    },
    sansio::DBusQueue,
};
use anyhow::{Result, bail, ensure};

pub(crate) trait SubscriptionResource {
    type Output: std::fmt::Debug;

    fn set_path(&mut self, path: String);
    fn try_process(&self, path: &str, body: Body<'_>) -> Result<Self::Output>;
}

pub(crate) struct Subscription<S>
where
    S: SubscriptionResource,
{
    path: Option<String>,
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

        let mut message: Message = RemoveMatch::new(&old_path).into();
        self.queue.push_back(&mut message);
    }

    fn subscribe(&mut self, sender: impl AsRef<str>, path: impl AsRef<str>) {
        let sender = sender.as_ref();
        let path = path.as_ref();
        let mut message: Message = AddMatch::new(sender, path).into();
        self.queue.push_back(&mut message);
        self.path = Some(path.to_string());
        self.resource.set_path(path.to_string());
    }

    pub(crate) fn start(&mut self, sender: impl AsRef<str>, path: impl AsRef<str>) {
        self.unsubscribe();
        self.subscribe(sender, path);
    }

    pub(crate) fn reset(&mut self) {
        self.unsubscribe()
    }

    fn try_process(&self, message: IncomingMessage<'_>) -> Result<S::Output> {
        ensure!(message.message_type == MessageType::Signal);

        let Some(interface) = message.interface else {
            bail!("no Interface")
        };
        ensure!(interface == "org.freedesktop.DBus.Properties");

        let Some(path) = message.path else {
            bail!("no Path")
        };

        let Some(body) = message.body else {
            bail!("no Body")
        };

        self.resource.try_process(path, body)
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

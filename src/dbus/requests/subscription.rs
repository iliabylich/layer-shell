use crate::{
    dbus::{
        OutgoingMessage,
        decoder::{Body, IncomingMessage, MessageType},
        messages::{
            interface_is,
            org_freedesktop_dbus::{AddMatch, RemoveMatch},
        },
    },
    sansio::{DBusConnectionKind, DBusQueue},
    utils::StringRef,
};
use anyhow::{Context as _, Result, bail, ensure};
use core::marker::PhantomData;

#[derive(Default, Clone)]
enum SubscriptionState {
    #[default]
    None,
    Subscribed(StringRef),
}

pub(crate) struct Subscription<T>
where
    T: 'static,
{
    try_process: &'static dyn Fn(Body<'_>, StringRef, StringRef) -> Result<T>,
    state: SubscriptionState,
    kind: DBusConnectionKind,
}

impl<T> Subscription<T> {
    fn unsubscribe(&mut self) {
        let SubscriptionState::Subscribed(path) = core::mem::take(&mut self.state) else {
            return;
        };

        let message: OutgoingMessage = RemoveMatch::new(path).into();
        DBusQueue::push_back(self.kind, message);
    }

    fn subscribe(&mut self, sender: StringRef, path: StringRef) {
        let message: OutgoingMessage = AddMatch::new(sender, path.clone()).into();
        DBusQueue::push_back(self.kind, message);
        self.state = SubscriptionState::Subscribed(path);
    }

    pub(crate) fn start(&mut self, sender: StringRef, path: StringRef) {
        self.unsubscribe();
        self.subscribe(sender, path);
    }

    pub(crate) fn reset(&mut self) {
        self.unsubscribe()
    }

    fn try_process(&self, message: IncomingMessage<'_>) -> Result<T> {
        ensure!(message.message_type == MessageType::Signal);

        let interface = message.interface.context("no Interface")?;
        interface_is!(interface, "org.freedesktop.DBus.Properties");
        let path = message.path.context("no Path")?;
        let body = message.body.context("no Body")?;

        let SubscriptionState::Subscribed(subscribed_to) = self.state.clone() else {
            bail!("not subscribed");
        };

        (self.try_process)(body, StringRef::new(path), subscribed_to)
    }

    pub(crate) fn process(&self, message: IncomingMessage<'_>) -> Option<T> {
        self.try_process(message).ok()
    }

    pub(crate) const fn builder() -> SubscriptionBuilder<T, NeedsTryProcess> {
        SubscriptionBuilder {
            try_process: &|_, _, _| todo!(),
            _state: PhantomData,
        }
    }
}

pub(crate) struct NeedsTryProcess;
pub(crate) struct NeedsConnectionKind;

pub(crate) struct SubscriptionBuilder<T, S>
where
    T: 'static,
{
    try_process: &'static dyn Fn(Body<'_>, StringRef, StringRef) -> Result<T>,
    _state: PhantomData<S>,
}
impl<T> SubscriptionBuilder<T, NeedsTryProcess> {
    pub(crate) const fn try_process(
        self,
        try_process: &'static dyn Fn(Body<'_>, StringRef, StringRef) -> Result<T>,
    ) -> SubscriptionBuilder<T, NeedsConnectionKind> {
        SubscriptionBuilder {
            try_process,
            _state: PhantomData,
        }
    }
}
impl<T> SubscriptionBuilder<T, NeedsConnectionKind> {
    pub(crate) const fn kind(self, kind: DBusConnectionKind) -> Subscription<T> {
        Subscription {
            try_process: self.try_process,
            state: SubscriptionState::None,
            kind,
        }
    }
}

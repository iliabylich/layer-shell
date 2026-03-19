use crate::{
    dbus::{
        Oneshot, OneshotResource, OutgoingMessage,
        decoder::{Body, IncomingMessage, Value},
        messages::{org_freedesktop_dbus::GetProperty, value_is},
    },
    ffi::ShortString,
    sansio::DBusQueue,
};
use anyhow::{Context, Result};

pub(crate) struct ActiveConnectionType {
    path: Option<ShortString>,
    oneshot: Oneshot<Resource>,
}

impl ActiveConnectionType {
    pub(crate) fn new(queue: DBusQueue) -> Self {
        Self {
            path: None,
            oneshot: Oneshot::new(Resource, queue),
        }
    }

    pub(crate) fn request(&mut self, path: ShortString) {
        self.oneshot.send(path);
        self.path = Some(path);
    }

    pub(crate) fn reset(&mut self) {
        self.oneshot.reset();
    }

    pub(crate) fn on_message(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<(bool, ShortString)> {
        let is_wireless = self.oneshot.try_rev(message).ok().flatten()?;
        Some((is_wireless, self.path?))
    }
}

struct Resource;
impl OneshotResource for Resource {
    type Input = ShortString;
    type Output = bool;

    fn request(&self, path: ShortString) -> impl Into<OutgoingMessage> {
        GetProperty::new(
            ShortString::new_const("org.freedesktop.NetworkManager"),
            path,
            ShortString::new_const("org.freedesktop.NetworkManager.Connection.Active"),
            ShortString::new_const("Type"),
        )
    }

    fn try_recv(&self, mut body: Body<'_>) -> Result<Self::Output> {
        let type_ = body.try_next()?.context("no Type in Body")?;
        value_is!(type_, Value::Variant(type_));
        let type_ = type_.materialize()?;
        value_is!(type_, Value::String(type_));

        Ok(type_.contains("wireless"))
    }
}

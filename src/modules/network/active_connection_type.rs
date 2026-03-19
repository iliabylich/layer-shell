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
    path: Option<String>,
    oneshot: Oneshot<Resource>,
}

impl ActiveConnectionType {
    pub(crate) fn new(queue: DBusQueue) -> Self {
        Self {
            path: None,
            oneshot: Oneshot::new(Resource, queue),
        }
    }

    pub(crate) fn request(&mut self, path: &str) {
        self.oneshot.start(path.to_string());
        self.path = Some(path.to_string());
    }

    pub(crate) fn reset(&mut self) {
        self.oneshot.reset();
    }

    pub(crate) fn on_message(&mut self, message: IncomingMessage<'_>) -> Option<(bool, String)> {
        let is_wireless = self.oneshot.process(message).ok().flatten()?;
        Some((is_wireless, self.path.clone()?))
    }
}

struct Resource;
impl OneshotResource for Resource {
    type Input = String;
    type Output = bool;

    fn make_request(&self, path: String) -> OutgoingMessage<'static> {
        GetProperty::new(
            ShortString::from("org.freedesktop.NetworkManager"),
            ShortString::from(path.as_str()),
            "org.freedesktop.NetworkManager.Connection.Active",
            "Type",
        )
        .into()
    }

    fn try_process(&self, mut body: Body<'_>) -> Result<Self::Output> {
        let type_ = body.try_next()?.context("no Type in Body")?;
        value_is!(type_, Value::Variant(type_));
        let type_ = type_.materialize()?;
        value_is!(type_, Value::String(type_));

        Ok(type_.contains("wireless"))
    }
}

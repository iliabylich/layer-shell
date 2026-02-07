use crate::dbus::{
    DBus, Message, Oneshot, OneshotResource,
    messages::{body_is, org_freedesktop_dbus::GetProperty, value_is},
    types::Value,
};
use anyhow::Result;

pub(crate) struct ActiveConnectionType {
    path: Option<String>,
    oneshot: Oneshot<Resource>,
}

impl ActiveConnectionType {
    pub(crate) fn new() -> Self {
        Self {
            path: None,
            oneshot: Oneshot::new(Resource),
        }
    }

    pub(crate) fn request(&mut self, dbus: &mut DBus, path: &str) {
        self.oneshot.start(dbus, path.to_string());
        self.path = Some(path.to_string());
    }

    pub(crate) fn reset(&mut self) {
        self.oneshot.reset();
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Option<(bool, String)> {
        let is_wireless = self.oneshot.process(message)?;
        Some((is_wireless, self.path.clone()?))
    }
}

struct Resource;
impl OneshotResource for Resource {
    type Input = String;
    type Output = bool;

    fn make_request(&self, path: String) -> Message<'static> {
        GetProperty::new(
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.Connection.Active",
            "Type",
        )
        .into()
    }

    fn try_process(&self, body: &[Value]) -> Result<Self::Output> {
        body_is!(body, [type_]);
        value_is!(type_, Value::Variant(type_));
        value_is!(&**type_, Value::String(type_));

        Ok(type_.contains("wireless"))
    }
}

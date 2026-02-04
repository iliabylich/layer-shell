use crate::{
    dbus::{
        DBus, Message, Oneshot, OneshotResource,
        messages::{body_is, interface_is, member_is, message_is, path_is, value_is},
        types::Value,
    },
    liburing::IoUring,
};
use anyhow::{Result, bail};
use std::borrow::Cow;

pub(crate) struct NameLostOrNameOwnerChanged {
    name_changed: Oneshot<NameOwnerChangedResource>,
}

impl NameLostOrNameOwnerChanged {
    pub(crate) fn new() -> Self {
        Self {
            name_changed: Oneshot::new(NameOwnerChangedResource),
        }
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus, ring: &mut IoUring) -> Result<()> {
        self.name_changed.start(dbus, (), ring)
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Option<String> {
        parse_name_owner_changed(message).ok()
    }
}

struct NameOwnerChangedResource;

impl OneshotResource for NameOwnerChangedResource {
    type Input = ();

    type Output = ();

    fn make_request(&self, _: Self::Input) -> Message<'static> {
        Message::MethodCall {
            serial: 0,
            path: Cow::Borrowed("/org/freedesktop/DBus"),
            member: Cow::Borrowed("AddMatch"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus")),
            destination: Some(Cow::Borrowed("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(Cow::Borrowed(
                "type='signal',sender='org.freedesktop.DBus',interface='org.freedesktop.DBus',member='NameOwnerChanged',path='/org/freedesktop/DBus'",
            ))],
        }
    }

    fn try_process(&self, _: &[Value]) -> Result<Self::Output> {
        unimplemented!()
    }
}

fn parse_name_owner_changed(message: &Message) -> Result<String> {
    message_is!(
        message,
        Message::Signal {
            path,
            interface,
            member,
            body,
            ..
        }
    );

    path_is!(path, "/org/freedesktop/DBus");
    interface_is!(interface, "org.freedesktop.DBus");
    member_is!(member, "NameOwnerChanged");
    body_is!(body, [alias, from, to]);

    value_is!(alias, Value::String(alias));
    value_is!(from, Value::String(_));
    value_is!(to, Value::String(to));

    if to.is_empty() {
        Ok(alias.to_string())
    } else {
        bail!("unrelated")
    }
}

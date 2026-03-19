use crate::{
    dbus::{
        Oneshot, OneshotResource, OutgoingMessage,
        decoder::{Body, IncomingMessage, MessageType, Value},
        messages::{interface_is, member_is, path_is, value_is},
    },
    ffi::ShortString,
    sansio::DBusQueue,
};
use anyhow::{Context, Result, bail, ensure};

pub(crate) struct NameLostOrNameOwnerChanged {
    name_changed: Oneshot<NameOwnerChangedResource>,
}

impl NameLostOrNameOwnerChanged {
    pub(crate) fn new(queue: DBusQueue) -> Self {
        Self {
            name_changed: Oneshot::new(NameOwnerChangedResource, queue),
        }
    }

    pub(crate) fn init(&mut self) {
        self.name_changed.send(())
    }

    pub(crate) fn on_message<'a>(&mut self, message: IncomingMessage<'a>) -> Option<ShortString> {
        let address = parse_name_owner_changed(message).ok()?;
        Some(ShortString::from(address))
    }
}

struct NameOwnerChangedResource;

impl OneshotResource for NameOwnerChangedResource {
    type Input = ();

    type Output = ();

    fn request(&self, _: Self::Input) -> impl Into<OutgoingMessage> {
        use crate::dbus::types::Value;
        OutgoingMessage::MethodCall {
            serial: 0,
            path: ShortString::new_const("/org/freedesktop/DBus"),
            member: ShortString::new_const("AddMatch"),
            interface: Some(ShortString::new_const("org.freedesktop.DBus")),
            destination: Some(ShortString::new_const("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::LongString(
                "type='signal',sender='org.freedesktop.DBus',interface='org.freedesktop.DBus',member='NameOwnerChanged',path='/org/freedesktop/DBus'".to_string(),
            )],
        }
    }

    fn try_recv(&self, _body: Body<'_>) -> Result<Self::Output> {
        unreachable!()
    }
}

fn parse_name_owner_changed<'a>(message: IncomingMessage<'a>) -> Result<&'a str> {
    ensure!(message.message_type == MessageType::Signal);

    let path = message.path.context("no Path")?;
    let interface = message.interface.context("no Interface")?;
    let member = message.member.context("no Member")?;
    let mut body = message.body.context("no Body")?;

    path_is!(path, "/org/freedesktop/DBus");
    interface_is!(interface, "org.freedesktop.DBus");
    member_is!(member, "NameOwnerChanged");

    let alias = body.try_next()?.context("no alias")?;
    let from = body.try_next()?.context("no from")?;
    let to = body.try_next()?.context("no to")?;

    value_is!(alias, Value::String(alias));
    value_is!(from, Value::String(_));
    value_is!(to, Value::String(to));

    if to.is_empty() {
        Ok(alias)
    } else {
        bail!("unrelated")
    }
}

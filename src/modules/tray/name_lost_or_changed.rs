use crate::{modules::SessionDBus, utils::StringRef};
use anyhow::{Context, Result, bail, ensure};
use mini_sansio_dbus::{
    IncomingMessage, IncomingValue, MessageType, MethodCall, OutgoingMessage, OutgoingValue,
    interface_is, member_is, path_is, value_is,
};

pub(crate) struct NameLostOrNameOwnerChanged {
    name_changed: MethodCall<(), (), ()>,
}

impl NameLostOrNameOwnerChanged {
    pub(crate) fn new() -> Self {
        Self {
            name_changed: SUBSCRIBE,
        }
    }

    pub(crate) fn init(&mut self) {
        self.name_changed.send((), SessionDBus::queue())
    }

    pub(crate) fn on_message<'a>(&mut self, message: IncomingMessage<'a>) -> Option<StringRef> {
        let address = parse_name_owner_changed(message).ok()?;
        Some(StringRef::new(address))
    }
}

const SUBSCRIBE: MethodCall<(), (), ()> = MethodCall::builder()
    .send(&|_input, _data| {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: String::from("/org/freedesktop/DBus"),
            member: String::from("AddMatch"),
            interface: Some(String::from("org.freedesktop.DBus")),
            destination: Some(String::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![OutgoingValue::String(
                "type='signal',sender='org.freedesktop.DBus',interface='org.freedesktop.DBus',member='NameOwnerChanged',path='/org/freedesktop/DBus'".to_string(),
            )],
        }
    }).try_process(&|_body, _data| unreachable!());

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

    value_is!(alias, IncomingValue::String(alias));
    value_is!(from, IncomingValue::String(_));
    value_is!(to, IncomingValue::String(to));

    if to.is_empty() {
        Ok(alias)
    } else {
        bail!("unrelated")
    }
}

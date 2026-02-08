use crate::dbus::{
    Message, OneshotResource,
    messages::{interface_is, member_is, message_is, path_is, sender_is},
    types::Value,
};
use anyhow::Result;
use std::borrow::Cow;

pub(crate) struct NewIconSubscription;

impl OneshotResource for NewIconSubscription {
    type Input = String;
    type Output = ();

    fn make_request(&self, address: Self::Input) -> Message<'static> {
        Message::MethodCall {
            destination: Some(Cow::Borrowed("org.freedesktop.DBus")),
            path: Cow::Borrowed("/org/freedesktop/DBus"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus")),
            serial: 0,
            member: Cow::Borrowed("AddMatch"),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(Cow::Owned(format!(
                "type='signal',sender='{address}',interface='org.kde.StatusNotifierItem',member='NewIcon',path='/StatusNotifierItem'"
            )))],
        }
    }

    fn try_process(&self, _body: &[Value]) -> Result<Self::Output> {
        Ok(())
    }
}

pub(crate) fn parse_new_icon_signal(message: &Message, address: &str) -> Result<()> {
    message_is!(
        message,
        Message::Signal {
            path,
            interface,
            member,
            sender: Some(sender),
            ..
        }
    );

    interface_is!(interface, "org.kde.StatusNotifierItem");
    path_is!(path, "/StatusNotifierItem");
    member_is!(member, "NewIcon");
    sender_is!(sender, address);

    Ok(())
}

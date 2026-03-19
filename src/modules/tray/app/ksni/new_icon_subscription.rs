use crate::{
    dbus::{
        OneshotResource, OutgoingMessage,
        decoder::{Body, IncomingMessage, MessageType},
        messages::{interface_is, member_is, path_is, sender_is},
        types::Value,
    },
    ffi::ShortString,
};
use anyhow::{Context as _, Result, ensure};

pub(crate) struct NewIconSubscription;

impl OneshotResource for NewIconSubscription {
    type Input = ShortString;
    type Output = ();

    fn make_request(&self, address: Self::Input) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            destination: Some(ShortString::new_const("org.freedesktop.DBus")),
            path: ShortString::new_const("/org/freedesktop/DBus"),
            interface: Some(ShortString::new_const("org.freedesktop.DBus")),
            serial: 0,
            member: ShortString::new_const("AddMatch"),
            sender: None,
            unix_fds: None,
            body: vec![Value::LongString(format!(
                "type='signal',sender='{address}',interface='org.kde.StatusNotifierItem',member='NewIcon',path='/StatusNotifierItem'"
            ))],
        }
    }

    fn try_process(&self, _body: Body<'_>) -> Result<Self::Output> {
        Ok(())
    }
}

pub(crate) fn parse_new_icon_signal(
    message: IncomingMessage<'_>,
    address: ShortString,
) -> Result<()> {
    ensure!(message.message_type == MessageType::Signal);

    let path = message.path.context("no Path")?;
    let interface = message.interface.context("no Interface")?;
    let member = message.member.context("no Member")?;
    let sender = message.sender.context("no Sender")?;

    interface_is!(interface, "org.kde.StatusNotifierItem");
    path_is!(path, "/StatusNotifierItem");
    member_is!(member, "NewIcon");
    sender_is!(sender, address);

    Ok(())
}

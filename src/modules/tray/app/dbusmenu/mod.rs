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
pub(crate) use get_layout::GetLayout;
use std::borrow::Cow;

mod get_layout;

pub(crate) struct LayoutUpdatedSubscription;

impl OneshotResource for LayoutUpdatedSubscription {
    type Input = (ShortString, ShortString);
    type Output = ();

    fn make_request(
        &self,
        (address, path): (ShortString, ShortString),
    ) -> OutgoingMessage<'static> {
        OutgoingMessage::MethodCall {
            destination: Some(ShortString::from("org.freedesktop.DBus")),
            path: Cow::Borrowed("/org/freedesktop/DBus"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus")),
            serial: 0,
            member: Cow::Borrowed("AddMatch"),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(Cow::Owned(format!(
                "type='signal',sender='{address}',interface='com.canonical.dbusmenu',member='LayoutUpdated',path='{path}'"
            )))],
        }
    }

    fn try_process(&self, _body: Body<'_>) -> Result<Self::Output> {
        Ok(())
    }
}

pub(crate) fn parse_layout_updated_signal(
    message: IncomingMessage<'_>,
    address: &str,
    expected_path: &str,
) -> Result<()> {
    ensure!(message.message_type == MessageType::Signal);

    let path = message.path.context("no Path")?;
    let interface = message.interface.context("no Interface")?;
    let member = message.member.context("no Member")?;
    let sender = message.sender.context("no Sender")?;

    interface_is!(interface, "com.canonical.dbusmenu");
    path_is!(path, expected_path);
    member_is!(member, "LayoutUpdated");
    sender_is!(sender, address);

    Ok(())
}

pub(crate) struct ItemsPropertiesUpdatedSubscription;

impl OneshotResource for ItemsPropertiesUpdatedSubscription {
    type Input = (ShortString, ShortString);
    type Output = ();

    fn make_request(
        &self,
        (address, path): (ShortString, ShortString),
    ) -> OutgoingMessage<'static> {
        OutgoingMessage::MethodCall {
            destination: Some(ShortString::from("org.freedesktop.DBus")),
            path: Cow::Borrowed("/org/freedesktop/DBus"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus")),
            serial: 0,
            member: Cow::Borrowed("AddMatch"),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(Cow::Owned(format!(
                "type='signal',sender='{address}',interface='com.canonical.dbusmenu',member='ItemsPropertiesUpdated',path='{path}'"
            )))],
        }
    }

    fn try_process(&self, _body: Body<'_>) -> Result<Self::Output> {
        Ok(())
    }
}

pub(crate) fn parse_items_properties_updated_signal(
    message: IncomingMessage<'_>,
    address: &str,
    expected_path: &str,
) -> Result<()> {
    ensure!(message.message_type == MessageType::Signal);

    let path = message.path.context("no Path")?;
    let interface = message.interface.context("no Interface")?;
    let member = message.member.context("no Member")?;
    let sender = message.sender.context("no Sender")?;

    interface_is!(interface, "com.canonical.dbusmenu");
    path_is!(path, expected_path);
    member_is!(member, "ItemsPropertiesUpdated");
    sender_is!(sender, address);

    Ok(())
}

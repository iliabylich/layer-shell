use crate::dbus::{
    Message, OneshotResource,
    messages::{interface_is, member_is, message_is, path_is, sender_is},
    types::Value,
};
use anyhow::Result;
pub(crate) use get_layout::GetLayout;
use std::borrow::Cow;

mod get_layout;

pub(crate) struct LayoutUpdatedSubscription;

impl OneshotResource for LayoutUpdatedSubscription {
    type Input = (String, String);
    type Output = ();

    fn make_request(&self, (address, path): (String, String)) -> Message<'static> {
        Message::MethodCall {
            destination: Some(Cow::Borrowed("org.freedesktop.DBus")),
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

    fn try_process(&self, _body: &[Value]) -> Result<Self::Output> {
        Ok(())
    }
}

pub(crate) fn parse_layout_updated_signal(
    message: &Message,
    address: &str,
    expected_path: &str,
) -> Result<()> {
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

    interface_is!(interface, "com.canonical.dbusmenu");
    path_is!(path, expected_path);
    member_is!(member, "LayoutUpdated");
    sender_is!(sender, address);

    Ok(())
}

pub(crate) struct ItemsPropertiesUpdatedSubscription;

impl OneshotResource for ItemsPropertiesUpdatedSubscription {
    type Input = (String, String);
    type Output = ();

    fn make_request(&self, (address, path): (String, String)) -> Message<'static> {
        Message::MethodCall {
            destination: Some(Cow::Borrowed("org.freedesktop.DBus")),
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

    fn try_process(&self, _body: &[Value]) -> Result<Self::Output> {
        Ok(())
    }
}

pub(crate) fn parse_items_properties_updated_signal(
    message: &Message,
    address: &str,
    expected_path: &str,
) -> Result<()> {
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

    interface_is!(interface, "com.canonical.dbusmenu");
    path_is!(path, expected_path);
    member_is!(member, "ItemsPropertiesUpdated");
    sender_is!(sender, address);

    Ok(())
}

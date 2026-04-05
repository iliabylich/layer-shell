use crate::{
    dbus::{
        MethodCall,
        decoder::{IncomingMessage, MessageType},
        messages::{interface_is, member_is, org_freedesktop_dbus::AddMatch, path_is, sender_is},
    },
    sansio::DBusConnectionKind,
    utils::StringRef,
};
use anyhow::{Context as _, Result, ensure};
pub(crate) use get_layout::GET_LAYOUT;

mod get_layout;

pub(crate) const SUBSCRIBE_TO_LAYOUT_UPDATED: MethodCall<(StringRef, StringRef), (), ()> =
    MethodCall::builder()
        .send(&|(address, path), _data| {
            AddMatch::build_from_rule(layout_updated_match_rule(address, path))
        })
        .try_process(&|_, _data| Ok(()))
        .kind(DBusConnectionKind::Session);

pub(crate) fn layout_updated_match_rule(address: StringRef, path: StringRef) -> String {
    format!(
        "type='signal',sender='{address}',interface='com.canonical.dbusmenu',member='LayoutUpdated',path='{path}'"
    )
}

pub(crate) fn parse_layout_updated_signal(
    message: IncomingMessage<'_>,
    address: StringRef,
    expected_path: StringRef,
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

pub(crate) const SUBSCRIBE_TO_ITEM_PROPERTIES_UPDATED: MethodCall<(StringRef, StringRef), (), ()> =
    MethodCall::builder()
        .send(&|(address, path), _data| {
            AddMatch::build_from_rule(items_properties_updated_match_rule(address, path))
        })
        .try_process(&|_body, _data| Ok(()))
        .kind(DBusConnectionKind::Session);

pub(crate) fn items_properties_updated_match_rule(address: StringRef, path: StringRef) -> String {
    format!(
        "type='signal',sender='{address}',interface='com.canonical.dbusmenu',member='ItemsPropertiesUpdated',path='{path}'"
    )
}

pub(crate) fn parse_items_properties_updated_signal(
    message: IncomingMessage<'_>,
    address: StringRef,
    expected_path: StringRef,
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

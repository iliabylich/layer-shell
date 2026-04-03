use crate::{
    dbus::{
        MethodCall,
        decoder::{IncomingMessage, MessageType},
        messages::{interface_is, member_is, org_freedesktop_dbus::AddMatch, path_is, sender_is},
    },
    ffi::ShortString,
    sansio::DBusConnectionKind,
};
use anyhow::{Context as _, Result, ensure};
pub(crate) use get_layout::GET_LAYOUT;

mod get_layout;

pub(crate) const SUBSCRIBE_TO_LAYOUT_UPDATED: MethodCall<(ShortString, ShortString), (), ()> =
    MethodCall::builder()
        .send(&|(address, path), _data| {
            AddMatch::from_rule(layout_updated_match_rule(address, path)).into()
        })
        .try_process(&|_, _data| Ok(()))
        .kind(DBusConnectionKind::Session);

pub(crate) fn layout_updated_match_rule(address: ShortString, path: ShortString) -> String {
    format!(
        "type='signal',sender='{address}',interface='com.canonical.dbusmenu',member='LayoutUpdated',path='{path}'"
    )
}

pub(crate) fn parse_layout_updated_signal(
    message: IncomingMessage<'_>,
    address: ShortString,
    expected_path: ShortString,
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

pub(crate) const SUBSCRIBE_TO_ITEM_PROPERTIES_UPDATED: MethodCall<
    (ShortString, ShortString),
    (),
    (),
> = MethodCall::builder()
    .send(&|(address, path), _data| {
        AddMatch::from_rule(items_properties_updated_match_rule(address, path)).into()
    })
    .try_process(&|_body, _data| Ok(()))
    .kind(DBusConnectionKind::Session);

pub(crate) fn items_properties_updated_match_rule(
    address: ShortString,
    path: ShortString,
) -> String {
    format!(
        "type='signal',sender='{address}',interface='com.canonical.dbusmenu',member='ItemsPropertiesUpdated',path='{path}'"
    )
}

pub(crate) fn parse_items_properties_updated_signal(
    message: IncomingMessage<'_>,
    address: ShortString,
    expected_path: ShortString,
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

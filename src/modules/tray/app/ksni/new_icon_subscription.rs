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

pub(crate) const SUBSCRIBE_TO_NEW_ICON: MethodCall<StringRef, (), ()> = MethodCall::builder()
    .send(&|address, _data| AddMatch::from_rule(new_icon_match_rule(address)).into())
    .try_process(&|_body, _data| Ok(()))
    .kind(DBusConnectionKind::Session);

pub(crate) fn new_icon_match_rule(address: StringRef) -> String {
    format!(
        "type='signal',sender='{address}',interface='org.kde.StatusNotifierItem',member='NewIcon',path='/StatusNotifierItem'"
    )
}

pub(crate) fn parse_new_icon_signal(
    message: IncomingMessage<'_>,
    address: StringRef,
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

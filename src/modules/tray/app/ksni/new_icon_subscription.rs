use crate::{
    dbus::{
        OneshotResource, OutgoingMessage,
        decoder::{Body, IncomingMessage, MessageType},
        messages::{interface_is, member_is, org_freedesktop_dbus::AddMatch, path_is, sender_is},
    },
    ffi::ShortString,
};
use anyhow::{Context as _, Result, ensure};

pub(crate) struct NewIconSubscription;

pub(crate) fn new_icon_match_rule(address: ShortString) -> String {
    format!(
        "type='signal',sender='{address}',interface='org.kde.StatusNotifierItem',member='NewIcon',path='/StatusNotifierItem'"
    )
}

impl OneshotResource for NewIconSubscription {
    type Input = ShortString;
    type Output = ();

    fn request(&self, address: Self::Input) -> impl Into<OutgoingMessage> {
        AddMatch::from_rule(new_icon_match_rule(address))
    }

    fn try_recv(&self, _body: Body<'_>) -> Result<Self::Output> {
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

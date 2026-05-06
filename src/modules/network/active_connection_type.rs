use crate::{modules::SystemDBus, utils::StringRef};
use anyhow::Context;
use mini_sansio_dbus::{
    IncomingBody, IncomingMessage, IncomingValue, IncompleteMethodCall, MethodCall,
    messages::org_freedesktop_dbus::GetProperty, value_is,
};

pub(crate) struct ActiveConnectionType {
    path: Option<StringRef>,
    oneshot: MethodCall<StringRef, bool, ()>,
}

impl ActiveConnectionType {
    pub(crate) fn new() -> Self {
        Self {
            path: None,
            oneshot: GET.with_data(()),
        }
    }

    pub(crate) fn request(&mut self, path: StringRef) {
        self.oneshot.send(path.clone(), SystemDBus::queue());
        self.path = Some(path);
    }

    pub(crate) const fn reset(&mut self) {
        self.oneshot.reset();
    }

    pub(crate) fn on_message(&mut self, message: IncomingMessage<'_>) -> Option<(bool, StringRef)> {
        let is_wireless = self.oneshot.try_recv(message).ok().flatten()?;
        Some((is_wireless, self.path.clone()?))
    }
}

const GET: IncompleteMethodCall<StringRef, bool, ()> =
    MethodCall::new(&|path: StringRef, _data| {
        GetProperty::build(
            "org.freedesktop.NetworkManager",
            path.as_str(),
            "org.freedesktop.NetworkManager.Connection.Active",
            "Type",
        )
    })
    .try_process(&|mut body: IncomingBody<'_>, _data| {
        let type_ = body.try_next()?.context("no Type in Body")?;
        value_is!(type_, IncomingValue::Variant(type_));
        let type_ = type_.materialize()?;
        value_is!(type_, IncomingValue::String(type_));

        Ok(type_.contains("wireless"))
    });

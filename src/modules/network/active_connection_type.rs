use crate::{
    dbus::{
        MethodCall,
        decoder::{IncomingMessage, Value},
        messages::{org_freedesktop_dbus::GetProperty, value_is},
    },
    ffi::ShortString,
    sansio::DBusConnectionKind,
};
use anyhow::Context;

pub(crate) struct ActiveConnectionType {
    path: Option<ShortString>,
    oneshot: MethodCall<ShortString, bool, ()>,
}

impl ActiveConnectionType {
    pub(crate) fn new() -> Self {
        Self {
            path: None,
            oneshot: GET,
        }
    }

    pub(crate) fn request(&mut self, path: ShortString) {
        self.oneshot.send(path);
        self.path = Some(path);
    }

    pub(crate) fn reset(&mut self) {
        self.oneshot.reset();
    }

    pub(crate) fn on_message(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<(bool, ShortString)> {
        let is_wireless = self.oneshot.try_recv(message).ok().flatten()?;
        Some((is_wireless, self.path?))
    }
}

const GET: MethodCall<ShortString, bool, ()> = MethodCall::builder()
    .send(&|path, _data| {
        GetProperty::new(
            ShortString::new_const("org.freedesktop.NetworkManager"),
            path,
            ShortString::new_const("org.freedesktop.NetworkManager.Connection.Active"),
            ShortString::new_const("Type"),
        )
        .into()
    })
    .try_process(&|mut body, _data| {
        let type_ = body.try_next()?.context("no Type in Body")?;
        value_is!(type_, Value::Variant(type_));
        let type_ = type_.materialize()?;
        value_is!(type_, Value::String(type_));

        Ok(type_.contains("wireless"))
    })
    .kind(DBusConnectionKind::System);

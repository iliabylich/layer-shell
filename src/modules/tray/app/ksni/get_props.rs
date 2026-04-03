use super::{AllPropsUpdate, parse};
use crate::{
    dbus::{
        OneshotMethodCall,
        decoder::Value,
        messages::{org_freedesktop_dbus::GetAllProperties, value_is},
    },
    ffi::ShortString,
    modules::TrayIcon,
    sansio::DBusConnectionKind,
};
use anyhow::{Context, bail};

pub(crate) const GET_MENU_AND_ICON: OneshotMethodCall<ShortString, (ShortString, TrayIcon), ()> = OneshotMethodCall::builder()
    .send(&|destination, _data| {
        GetAllProperties::new(
            destination,
            ShortString::new_const("/StatusNotifierItem"),
            ShortString::new_const("org.kde.StatusNotifierItem"),
        ).into()
    })
    .try_process(&|mut body, _data| {
        let array = body.try_next()?.context("no array")?;
        value_is!(array, Value::Array(array));
        match parse(array)? {
            AllPropsUpdate {
                menu: Some(menu),
                icon: Some(icon),
            } => Ok((menu, icon)),

            other => {
                log::error!(
                    "initial GetAllProps request for tray app failed, some data is missing: {other:?}"
                );
                bail!("DBus internal error")
            }
        }
    }).kind(DBusConnectionKind::Session);

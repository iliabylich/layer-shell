use crate::{
    dbus::{
        MethodCall, Subscription,
        decoder::{ArrayValue, Value},
        messages::{interface_is, org_freedesktop_dbus::GetAllProperties, path_is, value_is},
    },
    ffi::ShortString,
    modules::{TrayIcon, TrayIconPixmap},
    sansio::DBusConnectionKind,
};
use anyhow::{Context, Result, bail};

pub(crate) const GET_MENU_AND_ICON: MethodCall<ShortString, (ShortString, TrayIcon), ()> = MethodCall::builder()
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
            (Some(menu), Some(icon)) => Ok((menu, icon)),

            other => {
                log::error!(
                    "initial GetAllProps request for tray app failed, some data is missing: {other:?}"
                );
                bail!("DBus internal error")
            }
        }
    }).kind(DBusConnectionKind::Session);

pub(crate) const MENU_AND_ICON_SUBSCRIPTION: Subscription<(Option<ShortString>, Option<TrayIcon>)> =
    Subscription::builder()
        .try_process(&|mut body, path, _subscribed_to| {
            path_is!(path, "/StatusNotifierItem");

            let interface = body.try_next()?.context("no interface")?;
            value_is!(interface, Value::String(interface));
            interface_is!(interface, "org.kde.StatusNotifierItem");

            let items = body.try_next()?.context("no items")?;
            value_is!(items, Value::Array(items));
            parse(items)
        })
        .kind(DBusConnectionKind::Session);

fn parse(attributes: ArrayValue<'_>) -> Result<(Option<ShortString>, Option<TrayIcon>)> {
    let mut menu = None;
    let mut icon_name = None;
    let mut icon_pixmap = None;

    let mut iter = attributes.iter();
    while let Some(item) = iter.try_next()? {
        value_is!(item, Value::DictEntry(dict_entry));
        let (key, value) = dict_entry.key_value()?;
        value_is!(key, Value::String(key));
        value_is!(value, Value::Variant(value));

        match key {
            "Menu" => {
                let value = value.materialize()?;
                value_is!(value, Value::ObjectPath(value));
                menu = Some(value);
            }
            "IconName" => {
                let value = value.materialize()?;
                value_is!(value, Value::String(value));
                icon_name = Some(value);
            }
            "IconPixmap" => {
                let value = value.materialize()?;
                value_is!(value, Value::Array(value));

                let mut iter = value.iter();
                let Some(w_h_bytes) = iter.try_next()? else {
                    continue;
                };
                value_is!(w_h_bytes, Value::Struct(w_h_bytes));

                let mut iter = w_h_bytes.iter()?;

                let width = iter.try_next()?;
                let Some(width) = width else {
                    continue;
                };
                value_is!(width, Value::Int32(width));

                let height = iter.try_next()?.context("no height")?;
                value_is!(height, Value::Int32(height));

                let bytes = iter.try_next()?.context("no bytes")?;
                value_is!(bytes, Value::Array(bytes));

                let bytes = {
                    let mut out = vec![];
                    let mut iter = bytes.iter();
                    while let Some(byte) = iter.try_next()? {
                        value_is!(byte, Value::Byte(byte));
                        out.push(byte);
                    }
                    out
                };

                icon_pixmap = Some(TrayIconPixmap {
                    width,
                    height,
                    bytes: bytes.into(),
                });
            }
            _ => {}
        }
    }

    let icon = icon_name
        .and_then(|name_or_path| {
            if name_or_path.is_empty() {
                None
            } else {
                Some(TrayIcon::detect_name_or_path(name_or_path))
            }
        })
        .or_else(|| icon_pixmap.map(TrayIcon::Pixmap));

    Ok((menu.map(ShortString::from), icon))
}

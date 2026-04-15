use crate::{
    modules::{TrayIcon, TrayIconPixmap},
    utils::StringRef,
};
use anyhow::{Context, Result};
use mini_sansio_dbus::{
    IncomingArrayValue, IncomingValue, MethodCall, Subscription, interface_is,
    messages::org_freedesktop_dbus::GetAllProperties, path_is, value_is,
};

pub(crate) const GET_MENU_AND_ICON: MethodCall<StringRef, (StringRef, TrayIcon), ()> = MethodCall::builder()
    .send(&|destination: StringRef, _data| {
        GetAllProperties::build(
            destination.to_string(),
            "/StatusNotifierItem",
            "org.kde.StatusNotifierItem",
        )
    })
    .try_process(&|mut body, _data| {
        let array = body.try_next()?.context("no array")?;
        value_is!(array, IncomingValue::Array(array));
        match parse(array)? {
            (Some(menu), Some(icon)) => Ok((menu, icon)),

            other => {
                log::error!(
                    "initial GetAllProps request for tray app failed, some data is missing: {other:?}"
                );
                Err(anyhow::anyhow!("DBus internal error").into())
            }
        }
    });

pub(crate) const MENU_AND_ICON_SUBSCRIPTION: Subscription<(Option<StringRef>, Option<TrayIcon>)> =
    Subscription::new(&|mut body, path, _subscribed_to| {
        path_is!(path, "/StatusNotifierItem");

        let interface = body.try_next()?.context("no interface")?;
        value_is!(interface, IncomingValue::String(interface));
        interface_is!(interface, "org.kde.StatusNotifierItem");

        let items = body.try_next()?.context("no items")?;
        value_is!(items, IncomingValue::Array(items));
        parse(items).map_err(|err| err.into())
    });

fn parse(attributes: IncomingArrayValue<'_>) -> Result<(Option<StringRef>, Option<TrayIcon>)> {
    let mut menu = None;
    let mut icon_name = None;
    let mut icon_pixmap = None;

    let mut iter = attributes.iter();
    while let Some(item) = iter.try_next()? {
        value_is!(item, IncomingValue::DictEntry(dict_entry));
        let (key, value) = dict_entry.key_value()?;
        value_is!(key, IncomingValue::String(key));
        value_is!(value, IncomingValue::Variant(value));

        match key {
            "Menu" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::ObjectPath(value));
                menu = Some(value);
            }
            "IconName" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::String(value));
                icon_name = Some(value);
            }
            "IconPixmap" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::Array(value));

                let mut iter = value.iter();
                let Some(w_h_bytes) = iter.try_next()? else {
                    continue;
                };
                value_is!(w_h_bytes, IncomingValue::Struct(w_h_bytes));

                let mut iter = w_h_bytes.iter()?;

                let width = iter.try_next()?;
                let Some(width) = width else {
                    continue;
                };
                value_is!(width, IncomingValue::Int32(width));

                let height = iter.try_next()?.context("no height")?;
                value_is!(height, IncomingValue::Int32(height));

                let bytes = iter.try_next()?.context("no bytes")?;
                value_is!(bytes, IncomingValue::Array(bytes));

                let bytes = {
                    let mut out = vec![];
                    let mut iter = bytes.iter();
                    while let Some(byte) = iter.try_next()? {
                        value_is!(byte, IncomingValue::Byte(byte));
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

    Ok((menu.map(StringRef::new), icon))
}

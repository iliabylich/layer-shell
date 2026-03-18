use crate::{
    dbus::{
        SubscriptionResource,
        decoder::{ArrayValue, Body, Value},
        messages::{interface_is, path_is, value_is},
    },
    ffi::ShortString,
    modules::{TrayIcon, TrayIconPixmap},
};
use anyhow::{Context as _, Result};

pub(crate) struct AllPropsSubscription;

impl SubscriptionResource for AllPropsSubscription {
    type Output = AllPropsUpdate;

    fn set_path(&mut self, _path: String) {}

    fn try_process(&self, path: &str, mut body: Body<'_>) -> Result<Self::Output> {
        path_is!(path, "/StatusNotifierItem");

        let interface = body.try_next()?.context("no interface")?;
        value_is!(interface, Value::String(interface));
        interface_is!(interface, "org.kde.StatusNotifierItem");

        let items = body.try_next()?.context("no items")?;
        value_is!(items, Value::Array(items));
        parse(items)
    }
}

#[derive(Debug)]
pub(crate) struct AllPropsUpdate {
    pub(crate) menu: Option<ShortString>,
    pub(crate) icon: Option<TrayIcon>,
}

pub(crate) fn parse(attributes: ArrayValue<'_>) -> Result<AllPropsUpdate> {
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

    Ok(AllPropsUpdate {
        menu: menu.map(ShortString::from),
        icon,
    })
}

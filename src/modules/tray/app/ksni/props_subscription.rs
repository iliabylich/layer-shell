use crate::{
    dbus::{
        SubscriptionResource,
        messages::{interface_is, path_is, type_is, value_is},
        types::{CompleteType, Value},
    },
    modules::{TrayIcon, TrayIconPixmap},
};
use anyhow::{Result, anyhow};

pub(crate) struct AllPropsSubscription;

impl SubscriptionResource for AllPropsSubscription {
    type Output = AllPropsUpdate;

    fn set_path(&mut self, _path: String) {}

    fn try_process(&self, path: &str, interface: &str, items: &[Value]) -> Result<Self::Output> {
        path_is!(path, "/StatusNotifierItem");
        interface_is!(interface, "org.kde.StatusNotifierItem");

        parse(items)
    }
}

#[derive(Debug)]
pub(crate) struct AllPropsUpdate {
    pub(crate) menu: Option<String>,
    pub(crate) icon: Option<TrayIcon>,
}

pub(crate) fn parse(attributes: &[Value]) -> Result<AllPropsUpdate> {
    let mut menu = None;
    let mut icon_name = None;
    let mut icon_pixmap = None;

    for item in attributes {
        value_is!(item, Value::DictEntry(key, value));
        value_is!(&**key, Value::String(key));
        value_is!(&**value, Value::Variant(value));

        match key.as_ref() {
            "Menu" => {
                value_is!(&**value, Value::ObjectPath(value));
                menu = Some(value.to_string());
            }
            "IconName" => {
                value_is!(&**value, Value::String(value));
                icon_name = Some(value.to_string());
            }
            "IconPixmap" => {
                value_is!(&**value, Value::Array(CompleteType::Struct(item_t), value));

                let [item1_t, item2_t, item3_t]: &[CompleteType; 3] = item_t
                    .as_slice()
                    .try_into()
                    .map_err(|_| anyhow!("wrong IconPixmap size"))?;
                type_is!(item1_t, CompleteType::Int32);
                type_is!(item2_t, CompleteType::Int32);
                type_is!(item3_t, CompleteType::Array(item3_t));
                type_is!(&**item3_t, CompleteType::Byte);

                let Ok([w_h_bytes]): Result<&[Value; 1], _> = value.as_slice().try_into() else {
                    continue;
                };
                value_is!(w_h_bytes, Value::Struct(w_h_bytes));

                let [w, h, bytes]: &[Value; 3] = w_h_bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| anyhow!("wrong inner IconPixmap size"))?;

                value_is!(w, Value::Int32(w));
                value_is!(h, Value::Int32(h));
                value_is!(bytes, Value::Array(CompleteType::Byte, bytes));
                let bytes = bytes
                    .iter()
                    .map(|byte| {
                        value_is!(byte, Value::Byte(byte));
                        Ok(*byte)
                    })
                    .collect::<Result<Vec<_>>>()?;

                icon_pixmap = Some(TrayIconPixmap {
                    width: *w,
                    height: *h,
                    bytes: bytes.into(),
                });
            }
            _ => {}
        }
    }

    let icon = if let Some(name_or_path) = icon_name {
        if name_or_path.is_empty() {
            Some(TrayIcon::Unset)
        } else {
            Some(TrayIcon::detect_name_or_path(name_or_path))
        }
    } else if let Some(pixmap) = icon_pixmap {
        Some(TrayIcon::Pixmap(pixmap))
    } else {
        None
    };

    Ok(AllPropsUpdate { menu, icon })
}

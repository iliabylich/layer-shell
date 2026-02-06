use anyhow::{Context as _, Result, anyhow, bail, ensure};

use crate::{
    dbus::{
        DBus, Message, Oneshot, OneshotResource,
        messages::{body_is, org_freedesktop_dbus::GetAllProperties, type_is, value_is},
        types::{CompleteType, Value},
    },
    liburing::IoUring,
};

pub(crate) struct TrayApp {
    address: String,
    oneshot: Oneshot<GetAllProps>,
}

impl TrayApp {
    pub(crate) fn new(name: String) -> Self {
        Self {
            address: name,
            oneshot: Oneshot::new(GetAllProps),
        }
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus, ring: &mut IoUring) -> Result<()> {
        self.oneshot.start(dbus, self.address.clone(), ring)?;
        Ok(())
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Result<()> {
        if let Some(t) = self.oneshot.process(message) {
            println!("{t:?}");
        }
        Ok(())
    }
}

#[derive(Debug)]
struct IconPixmap {
    w: i32,
    h: i32,
    bytes: Vec<u8>,
}

struct GetAllProps;

#[derive(Debug)]
struct GetAllPropsOutput {
    menu: Option<String>,
    icon_name: Option<String>,
    icon_pixmap: Option<IconPixmap>,
}

impl OneshotResource for GetAllProps {
    type Input = String;
    type Output = GetAllPropsOutput;

    fn make_request(&self, input: Self::Input) -> Message<'static> {
        GetAllProperties::new(input, "/StatusNotifierItem", "org.kde.StatusNotifierItem").into()
    }

    fn try_process(&self, body: &[Value]) -> Result<Self::Output> {
        body_is!(body, [Value::Array(item_t, array)]);
        type_is!(item_t, CompleteType::DictEntry(key_t, value_t));
        type_is!(&**key_t, CompleteType::String);
        type_is!(&**value_t, CompleteType::Variant);

        let mut menu = None;
        let mut icon_name = None;
        let mut icon_pixmap = None;

        for item in array {
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

                    let Ok([w_h_bytes]): Result<&[Value; 1], _> = value.as_slice().try_into()
                    else {
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

                    icon_pixmap = Some(IconPixmap {
                        w: *w,
                        h: *h,
                        bytes,
                    });
                }
                _ => {}
            }
        }

        Ok(GetAllPropsOutput {
            menu,
            icon_name,
            icon_pixmap,
        })
    }
}

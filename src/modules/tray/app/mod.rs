use crate::dbus::{
    DBus, Message, Oneshot, OneshotResource, Subscription, SubscriptionResource,
    messages::{
        body_is, interface_is, org_freedesktop_dbus::GetAllProperties, path_is, type_is, value_is,
    },
    types::{CompleteType, Value},
};
use anyhow::{Result, anyhow};
use std::borrow::Cow;

pub(crate) struct App {
    address: String,
    get_all_props: Oneshot<GetAllPropsOneshot>,
    new_icon: Oneshot<NewIcon>,
    subscription: Subscription<AllPropsSubscription>,
}

impl App {
    pub(crate) fn new(name: String) -> Self {
        Self {
            address: name,
            get_all_props: Oneshot::new(GetAllPropsOneshot),
            new_icon: Oneshot::new(NewIcon),
            subscription: Subscription::new(AllPropsSubscription),
        }
    }

    fn request_props(&mut self, dbus: &mut DBus) {
        self.new_icon = Oneshot::new(NewIcon);
        self.new_icon.start(dbus, self.address.clone());
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        self.request_props(dbus);
        self.get_all_props.start(dbus, self.address.clone());
        self.subscription.start(dbus, "/StatusNotifierItem");
    }

    pub(crate) fn on_message(&mut self, message: &Message, dbus: &mut DBus) {
        if let Some(_) = self.get_all_props.process(message) {
            log::error!("GOT ONESHOT PROPS FOR {}", self.address);
        }
        if let Some(()) = self.new_icon.process(message) {
            self.request_props(dbus);
        }
        if let Some(_) = self.subscription.process(message) {
            log::error!("GOT SUBSCRIPTION PROPS for {}", self.address)
        }
    }
}

#[derive(Debug)]
struct IconPixmap {
    w: i32,
    h: i32,
    bytes: Vec<u8>,
}

struct GetAllPropsOneshot;

#[derive(Debug)]
struct AllProps {
    menu: Option<String>,
    icon_name: Option<String>,
    icon_pixmap: Option<IconPixmap>,
}

impl OneshotResource for GetAllPropsOneshot {
    type Input = String;
    type Output = AllProps;

    fn make_request(&self, input: Self::Input) -> Message<'static> {
        GetAllProperties::new(input, "/StatusNotifierItem", "org.kde.StatusNotifierItem").into()
    }

    fn try_process(&self, body: &[Value]) -> Result<Self::Output> {
        body_is!(body, [Value::Array(item_t, array)]);
        type_is!(item_t, CompleteType::DictEntry(key_t, value_t));
        type_is!(&**key_t, CompleteType::String);
        type_is!(&**value_t, CompleteType::Variant);

        parse(array)
    }
}

struct AllPropsSubscription;

impl SubscriptionResource for AllPropsSubscription {
    type Output = AllProps;

    fn set_path(&mut self, _path: String) {}

    fn try_process(&self, path: &str, interface: &str, items: &[Value]) -> Result<Self::Output> {
        path_is!(path, "/StatusNotifierItem");
        interface_is!(interface, "org.kde.StatusNotifierItem");

        parse(items)
    }
}

fn parse(attributes: &[Value]) -> Result<AllProps> {
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

                icon_pixmap = Some(IconPixmap {
                    w: *w,
                    h: *h,
                    bytes,
                });
            }
            _ => {}
        }
    }

    Ok(AllProps {
        menu,
        icon_name,
        icon_pixmap,
    })
}

struct NewIcon;

impl OneshotResource for NewIcon {
    type Input = String;
    type Output = ();

    fn make_request(&self, address: Self::Input) -> Message<'static> {
        Message::MethodCall {
            destination: Some(Cow::Borrowed("org.freedesktop.DBus")),
            path: Cow::Borrowed("/org/freedesktop/DBus"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus")),
            serial: 0,
            member: Cow::Borrowed("AddMatch"),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(Cow::Owned(format!(
                "type='signal',sender='{address}',interface='org.kde.StatusNotifierItem',member='NewIcon',path='/StatusNotifierItem'"
            )))],
        }
    }

    fn try_process(&self, _body: &[Value]) -> Result<Self::Output> {
        Ok(())
    }
}

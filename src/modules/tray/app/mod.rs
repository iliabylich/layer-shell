use crate::{
    dbus::{
        DBus, Message, Oneshot, OneshotResource, Subscription, SubscriptionResource,
        messages::{
            body_is, interface_is, org_freedesktop_dbus::GetAllProperties, path_is, type_is,
            value_is,
        },
        types::{CompleteType, Value},
    },
    modules::{TrayIcon, TrayIconPixmap, TrayItem},
};
use anyhow::{Result, anyhow, bail};
use get_layout::GetLayout;
use std::borrow::Cow;

mod get_layout;

pub(crate) struct App {
    address: String,
    all_props_request: Oneshot<GetAllPropsOneshot>,
    new_icon_subscription: Oneshot<NewIconSubscription>,
    all_props_subscription: Subscription<AllPropsSubscription>,

    menu: String,
    get_layout: Oneshot<GetLayout>,

    state: IconLayoutState,
}

enum IconLayoutState {
    HaveNothing,
    HaveOnlyIcon(TrayIcon),
    HaveOnlyLayout(Vec<TrayItem>),
    HaveAll,
}

#[derive(Debug)]
pub(crate) enum TrayEvent {
    Initialized(TrayIcon, Vec<TrayItem>),
    IconUpdated(TrayIcon),
    MenuUpdated(Vec<TrayItem>),
}

impl App {
    pub(crate) fn new(name: String) -> Self {
        Self {
            address: name.clone(),
            all_props_request: Oneshot::new(GetAllPropsOneshot),
            new_icon_subscription: Oneshot::new(NewIconSubscription),
            all_props_subscription: Subscription::new(AllPropsSubscription),

            menu: String::new(),
            get_layout: Oneshot::new(GetLayout::new(name)),

            state: IconLayoutState::HaveNothing,
        }
    }

    fn request_props(&mut self, dbus: &mut DBus) {
        self.new_icon_subscription = Oneshot::new(NewIconSubscription);
        self.new_icon_subscription.start(dbus, self.address.clone());
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        self.request_props(dbus);
        self.all_props_request.start(dbus, self.address.clone());
        self.all_props_subscription
            .start(dbus, "/StatusNotifierItem");
    }

    pub(crate) fn reset(&mut self, dbus: &mut DBus) {
        self.new_icon_subscription.reset();
        self.all_props_request.reset();
        self.all_props_subscription.reset(dbus);
        self.get_layout.reset();
    }

    fn schedule_get_layout(&mut self, dbus: &mut DBus) {
        let mut get_layout = Oneshot::new(GetLayout::new(&self.address));
        get_layout.start(dbus, (self.address.clone(), self.menu.clone()));
        self.get_layout = get_layout;
    }

    fn on_menu_received(&mut self, menu: String, dbus: &mut DBus) {
        self.menu = menu;
        self.schedule_get_layout(dbus);
    }

    fn on_icon_received(&mut self, new_icon: TrayIcon) -> Option<TrayEvent> {
        match &mut self.state {
            IconLayoutState::HaveNothing => {
                self.state = IconLayoutState::HaveOnlyIcon(new_icon);
                None
            }
            IconLayoutState::HaveOnlyIcon(icon) => {
                *icon = new_icon;
                None
            }
            IconLayoutState::HaveOnlyLayout(layout) => {
                let layout = std::mem::take(layout);
                self.state = IconLayoutState::HaveAll;
                Some(TrayEvent::Initialized(new_icon, layout))
            }
            IconLayoutState::HaveAll => Some(TrayEvent::IconUpdated(new_icon)),
        }
    }

    fn on_layout_receieved(&mut self, new_layout: Vec<TrayItem>) -> Option<TrayEvent> {
        match &mut self.state {
            IconLayoutState::HaveNothing => {
                self.state = IconLayoutState::HaveOnlyLayout(new_layout);
                None
            }
            IconLayoutState::HaveOnlyIcon(icon) => {
                let icon = std::mem::take(icon);
                self.state = IconLayoutState::HaveAll;
                Some(TrayEvent::Initialized(icon, new_layout))
            }
            IconLayoutState::HaveOnlyLayout(layout) => {
                *layout = new_layout;
                None
            }
            IconLayoutState::HaveAll => Some(TrayEvent::MenuUpdated(new_layout)),
        }
    }

    pub(crate) fn on_message(&mut self, message: &Message, dbus: &mut DBus) -> Option<TrayEvent> {
        if let Some(AllProps { menu, icon }) = self.all_props_request.process(message) {
            log::error!("GOT ONESHOT PROPS FOR {}", self.address);

            self.on_menu_received(menu, dbus);
            return self.on_icon_received(icon);
        }

        if let Some(()) = self.new_icon_subscription.process(message) {
            self.request_props(dbus);
        }

        if let Some(AllPropsUpdate { icon, .. }) = self.all_props_subscription.process(message) {
            log::error!("GOT SUBSCRIPTION PROPS for {}", self.address);
            if let Some(icon) = icon {
                return self.on_icon_received(icon);
            }
        }

        if let Some(layout) = self.get_layout.process(message) {
            log::error!("Got layout");
            return self.on_layout_receieved(layout);
        }

        None
    }
}

struct GetAllPropsOneshot;

#[derive(Debug)]
struct AllProps {
    menu: String,
    icon: TrayIcon,
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

        match parse(array)? {
            AllPropsUpdate {
                menu: Some(menu),
                icon: Some(icon),
            } => Ok(AllProps { menu, icon }),

            other => {
                log::error!(
                    "initial GetAllProps request for tray app failed, some data is missing: {other:?}"
                );
                bail!("DBus internal error")
            }
        }
    }
}

struct AllPropsSubscription;

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
struct AllPropsUpdate {
    menu: Option<String>,
    icon: Option<TrayIcon>,
}

fn parse(attributes: &[Value]) -> Result<AllPropsUpdate> {
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

struct NewIconSubscription;

impl OneshotResource for NewIconSubscription {
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

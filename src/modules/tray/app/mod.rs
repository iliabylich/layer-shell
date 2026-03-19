use crate::{
    dbus::{Oneshot, OutgoingMessage, Subscription, decoder::IncomingMessage, types::Value},
    ffi::ShortString,
    macros::report_and_exit,
    modules::{TrayIcon, TrayItem, tray::service::Service},
    sansio::DBusQueue,
};
use dbusmenu::{
    GetLayout, ItemsPropertiesUpdatedSubscription, LayoutUpdatedSubscription,
    parse_items_properties_updated_signal, parse_layout_updated_signal,
};
use ksni::{
    AllProps, AllPropsSubscription, AllPropsUpdate, GetAllPropsOneshot, NewIconSubscription,
    parse_new_icon_signal,
};
use std::borrow::Cow;

mod dbusmenu;
mod ksni;

pub(crate) struct App {
    service: Service,

    all_props_request: Oneshot<GetAllPropsOneshot>,
    new_icon_subscription: Oneshot<NewIconSubscription>,
    all_props_subscription: Subscription<AllPropsSubscription>,
    layout_updated_subscription: Oneshot<LayoutUpdatedSubscription>,
    items_properties_updated_subscription: Oneshot<ItemsPropertiesUpdatedSubscription>,

    menu: ShortString,
    get_layout: Oneshot<GetLayout>,

    state: State,
    queue: DBusQueue,
}

enum State {
    Nothing,
    OnlyIcon(TrayIcon),
    OnlyLayout(Vec<TrayItem>),
    All,
}

#[derive(Debug)]
pub(crate) enum TrayEvent {
    Initialized(TrayIcon, Vec<TrayItem>),
    IconUpdated(TrayIcon),
    MenuUpdated(Vec<TrayItem>),
}

impl App {
    pub(crate) fn new(service: Service, queue: DBusQueue) -> Self {
        Self {
            service,
            all_props_request: Oneshot::new(GetAllPropsOneshot, queue.copy()),
            new_icon_subscription: Oneshot::new(NewIconSubscription, queue.copy()),
            all_props_subscription: Subscription::new(AllPropsSubscription, queue.copy()),
            layout_updated_subscription: Oneshot::new(LayoutUpdatedSubscription, queue.copy()),
            items_properties_updated_subscription: Oneshot::new(
                ItemsPropertiesUpdatedSubscription,
                queue.copy(),
            ),

            menu: ShortString::from(""),
            get_layout: Oneshot::new(GetLayout::new(service.name()), queue.copy()),

            state: State::Nothing,
            queue,
        }
    }

    fn schedule_request_props(&mut self) {
        self.all_props_request.reset();
        self.all_props_request = Oneshot::new(GetAllPropsOneshot, self.queue.copy());
        self.all_props_request.start(self.service.name());
    }

    pub(crate) fn init(&mut self) {
        self.new_icon_subscription = Oneshot::new(NewIconSubscription, self.queue.copy());
        self.new_icon_subscription.start(self.service.name());
        self.all_props_request.start(self.service.name());
        self.all_props_subscription
            .start("org.freedesktop.DBus", "/StatusNotifierItem");
    }

    pub(crate) fn reset(&mut self) {
        self.new_icon_subscription.reset();
        self.all_props_request.reset();
        self.all_props_subscription.reset();
        self.get_layout.reset();
    }

    fn schedule_get_layout(&mut self) {
        let mut get_layout = Oneshot::new(GetLayout::new(self.service.name()), self.queue.copy());
        get_layout.start((self.service.name(), self.menu));
        self.get_layout = get_layout;
    }

    fn on_menu_received(&mut self, menu: ShortString) {
        if !self.menu.as_str().is_empty() {
            return;
        }

        self.menu = menu;
        self.schedule_get_layout();
        self.layout_updated_subscription
            .start((self.service.name(), self.menu));
        self.items_properties_updated_subscription
            .start((self.service.name(), self.menu));
    }

    fn on_icon_received(&mut self, new_icon: TrayIcon) -> Option<TrayEvent> {
        match &mut self.state {
            State::Nothing => {
                self.state = State::OnlyIcon(new_icon);
                None
            }
            State::OnlyIcon(icon) => {
                *icon = new_icon;
                None
            }
            State::OnlyLayout(layout) => {
                let layout = std::mem::take(layout);
                self.state = State::All;
                Some(TrayEvent::Initialized(new_icon, layout))
            }
            State::All => Some(TrayEvent::IconUpdated(new_icon)),
        }
    }

    fn on_layout_receieved(&mut self, new_layout: Vec<TrayItem>) -> Option<TrayEvent> {
        match &mut self.state {
            State::Nothing => {
                self.state = State::OnlyLayout(new_layout);
                None
            }
            State::OnlyIcon(icon) => {
                let icon = std::mem::take(icon);
                self.state = State::All;
                Some(TrayEvent::Initialized(icon, new_layout))
            }
            State::OnlyLayout(layout) => {
                *layout = new_layout;
                None
            }
            State::All => Some(TrayEvent::MenuUpdated(new_layout)),
        }
    }

    pub(crate) fn on_message(&mut self, message: IncomingMessage<'_>) -> Option<TrayEvent> {
        if let Some(AllProps { menu, icon }) =
            self.all_props_request.process(message).ok().flatten()
        {
            log::info!(target: "Tray", "Received requested props for {:?}", self.service);

            self.on_menu_received(menu);
            return self.on_icon_received(icon);
        }

        if let Some(()) = self.new_icon_subscription.process(message).ok().flatten() {
            log::info!(target: "Tray", "Subscribed to NewIcon");
            return None;
        }

        if let Some(AllPropsUpdate { icon, .. }) = self.all_props_subscription.process(message) {
            log::info!(target: "Tray", "Received updated props for {:?}", self.service);
            if let Some(icon) = icon {
                return self.on_icon_received(icon);
            }
        }

        if let Some(layout) = self.get_layout.process(message).ok().flatten() {
            log::info!(target: "Tray", "Got layout");
            return self.on_layout_receieved(layout);
        }

        if self
            .layout_updated_subscription
            .process(message)
            .ok()
            .flatten()
            .is_some()
        {
            log::info!(target: "Tray", "Subscribed to LayoutUpdated");
            return None;
        }

        if self
            .items_properties_updated_subscription
            .process(message)
            .ok()
            .flatten()
            .is_some()
        {
            log::info!(target: "Tray", "Subscribed to ItemPropertiesUpdated");
            return None;
        }

        if parse_new_icon_signal(message, self.service.raw_address().as_str()).is_ok() {
            log::info!(target: "Tray", "Received NewIcon signal");
            self.schedule_request_props();
            return None;
        }

        if parse_layout_updated_signal(
            message,
            self.service.raw_address().as_str(),
            self.menu.as_str(),
        )
        .is_ok()
        {
            log::info!(target: "Tray", "Received LayoutUpdated signal");
            self.schedule_get_layout();
            return None;
        }
        if parse_items_properties_updated_signal(
            message,
            self.service.raw_address().as_str(),
            self.menu.as_str(),
        )
        .is_ok()
        {
            log::info!(target: "Tray", "Received ItemsPropertiesUpdated signal");
            self.schedule_get_layout();
            return None;
        }

        None
    }

    pub(crate) fn trigger(&self, id: i32) {
        let timestamp = u32::try_from(chrono::Utc::now().timestamp()).unwrap_or_else(|err| {
            report_and_exit!(target: "Tray", "can't construct u32 from chrono timestamp: {err:?}")
        });

        let mut message = OutgoingMessage::MethodCall {
            destination: Some(self.service.name()),
            path: Cow::Owned(self.menu.to_string()),
            interface: Some(Cow::Borrowed("com.canonical.dbusmenu")),
            serial: 0,
            member: Cow::Borrowed("Event"),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::Int32(id),
                Value::String(Cow::Borrowed("clicked")),
                Value::Variant(Box::new(Value::Int32(0))),
                Value::UInt32(timestamp),
            ],
        };

        self.queue.push_back(&mut message);
    }
}

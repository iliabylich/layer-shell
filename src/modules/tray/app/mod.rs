use crate::{
    dbus::{
        MethodCall, OutgoingMessage, Subscription, decoder::IncomingMessage,
        messages::org_freedesktop_dbus::RemoveMatch, types::Value,
    },
    ffi::ShortString,
    modules::{TrayIcon, TrayItem, tray::service::Service},
    sansio::SessionDBusQueue,
    utils::report_and_exit,
};
use dbusmenu::{
    GET_LAYOUT, SUBSCRIBE_TO_ITEM_PROPERTIES_UPDATED, SUBSCRIBE_TO_LAYOUT_UPDATED,
    items_properties_updated_match_rule, layout_updated_match_rule,
    parse_items_properties_updated_signal, parse_layout_updated_signal,
};
use ksni::{
    GET_MENU_AND_ICON, MENU_AND_ICON_SUBSCRIPTION, SUBSCRIBE_TO_NEW_ICON, new_icon_match_rule,
    parse_new_icon_signal,
};

mod dbusmenu;
mod ksni;

pub(crate) struct App {
    service: Service,

    get_menu_and_icon: MethodCall<ShortString, (ShortString, TrayIcon), ()>,
    subscribe_to_new_icon: MethodCall<ShortString, (), ()>,
    menu_and_icon_subscription: Subscription<(Option<ShortString>, Option<TrayIcon>)>,
    subscribe_to_layout_updated: MethodCall<(ShortString, ShortString), (), ()>,
    subscribe_to_items_properties_updated: MethodCall<(ShortString, ShortString), (), ()>,

    menu: ShortString,
    get_layout: MethodCall<(ShortString, ShortString), Vec<TrayItem>, ShortString>,

    state: State,
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
    pub(crate) fn new(service: Service) -> Self {
        Self {
            service,
            get_menu_and_icon: GET_MENU_AND_ICON,
            subscribe_to_new_icon: SUBSCRIBE_TO_NEW_ICON,
            menu_and_icon_subscription: MENU_AND_ICON_SUBSCRIPTION,
            subscribe_to_layout_updated: SUBSCRIBE_TO_LAYOUT_UPDATED,
            subscribe_to_items_properties_updated: SUBSCRIBE_TO_ITEM_PROPERTIES_UPDATED,

            menu: ShortString::new_const(""),
            get_layout: GET_LAYOUT.with_data(service.name()),

            state: State::Nothing,
        }
    }

    fn schedule_request_props(&mut self) {
        self.get_menu_and_icon.reset();
        self.get_menu_and_icon.send(self.service.name());
    }

    pub(crate) fn init(&mut self) {
        self.subscribe_to_new_icon.send(self.service.name());
        self.get_menu_and_icon.send(self.service.name());
        self.menu_and_icon_subscription.start(
            ShortString::new_const("org.freedesktop.DBus"),
            ShortString::new_const("/StatusNotifierItem"),
        );
    }

    pub(crate) fn reset(&mut self) {
        self.unsubscribe_matches();
        self.subscribe_to_new_icon.reset();
        self.get_menu_and_icon.reset();
        self.menu_and_icon_subscription.reset();
        self.get_layout.reset();
    }

    fn remove_match(&self, rule: String) {
        let message: OutgoingMessage = RemoveMatch::from_rule(rule).into();
        SessionDBusQueue::push_back(message);
    }

    fn unsubscribe_matches(&self) {
        self.remove_match(new_icon_match_rule(self.service.name()));

        if self.menu != "" {
            self.remove_match(layout_updated_match_rule(self.service.name(), self.menu));
            self.remove_match(items_properties_updated_match_rule(
                self.service.name(),
                self.menu,
            ));
        }
    }

    fn schedule_get_layout(&mut self) {
        self.get_layout.reset();
        self.get_layout.send((self.service.name(), self.menu));
    }

    fn on_menu_received(&mut self, menu: ShortString) {
        if self.menu != "" {
            return;
        }

        self.menu = menu;
        self.schedule_get_layout();
        self.subscribe_to_layout_updated
            .send((self.service.name(), self.menu));
        self.subscribe_to_items_properties_updated
            .send((self.service.name(), self.menu));
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
                let layout = core::mem::take(layout);
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
                let icon = core::mem::take(icon);
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
        if let Some((menu, icon)) = self.get_menu_and_icon.try_recv(message).ok().flatten() {
            log::info!(target: "Tray", "Received requested props for {:?}", self.service);

            self.on_menu_received(menu);
            return self.on_icon_received(icon);
        }

        if let Some(()) = self.subscribe_to_new_icon.try_recv(message).ok().flatten() {
            log::info!(target: "Tray", "Subscribed to NewIcon");
            return None;
        }

        if let Some((_, icon)) = self.menu_and_icon_subscription.process(message) {
            log::info!(target: "Tray", "Received updated props for {:?}", self.service);
            if let Some(icon) = icon {
                return self.on_icon_received(icon);
            }
        }

        if let Some(layout) = self.get_layout.try_recv(message).ok().flatten() {
            log::info!(target: "Tray", "Got layout");
            return self.on_layout_receieved(layout);
        }

        if self
            .subscribe_to_layout_updated
            .try_recv(message)
            .ok()
            .flatten()
            .is_some()
        {
            log::info!(target: "Tray", "Subscribed to LayoutUpdated");
            return None;
        }

        if self
            .subscribe_to_items_properties_updated
            .try_recv(message)
            .ok()
            .flatten()
            .is_some()
        {
            log::info!(target: "Tray", "Subscribed to ItemPropertiesUpdated");
            return None;
        }

        if parse_new_icon_signal(message, self.service.raw_address()).is_ok() {
            log::info!(target: "Tray", "Received NewIcon signal");
            self.schedule_request_props();
            return None;
        }

        if parse_layout_updated_signal(message, self.service.raw_address(), self.menu).is_ok() {
            log::info!(target: "Tray", "Received LayoutUpdated signal");
            self.schedule_get_layout();
            return None;
        }
        if parse_items_properties_updated_signal(message, self.service.raw_address(), self.menu)
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

        let message = OutgoingMessage::MethodCall {
            destination: Some(self.service.name()),
            path: self.menu,
            interface: Some(ShortString::new_const("com.canonical.dbusmenu")),
            serial: 0,
            member: ShortString::new_const("Event"),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::Int32(id),
                Value::ShortString(ShortString::new_const("clicked")),
                Value::Variant(Box::new(Value::Int32(0))),
                Value::UInt32(timestamp),
            ],
        };

        SessionDBusQueue::push_back(message);
    }
}

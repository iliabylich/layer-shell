use crate::{
    dbus::{DBus, Message, Oneshot, Subscription},
    modules::{TrayIcon, TrayItem, tray::service::Service},
};
use dbusmenu::{
    GetLayout, ItemsPropertiesUpdatedSubscription, LayoutUpdatedSubscription,
    parse_items_properties_updated_signal, parse_layout_updated_signal,
};
use ksni::{
    AllProps, AllPropsSubscription, AllPropsUpdate, GetAllPropsOneshot, NewIconSubscription,
    parse_new_icon_signal,
};

mod dbusmenu;
mod ksni;

pub(crate) struct App {
    service: Service,

    all_props_request: Oneshot<GetAllPropsOneshot>,
    new_icon_subscription: Oneshot<NewIconSubscription>,
    all_props_subscription: Subscription<AllPropsSubscription>,
    layout_updated_subscription: Oneshot<LayoutUpdatedSubscription>,
    items_properties_updated_subscription: Oneshot<ItemsPropertiesUpdatedSubscription>,

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
    pub(crate) fn new(service: Service) -> Self {
        Self {
            service: service.clone(),
            all_props_request: Oneshot::new(GetAllPropsOneshot),
            new_icon_subscription: Oneshot::new(NewIconSubscription),
            all_props_subscription: Subscription::new(AllPropsSubscription),
            layout_updated_subscription: Oneshot::new(LayoutUpdatedSubscription),
            items_properties_updated_subscription: Oneshot::new(ItemsPropertiesUpdatedSubscription),

            menu: String::new(),
            get_layout: Oneshot::new(GetLayout::new(service.name())),

            state: IconLayoutState::HaveNothing,
        }
    }

    fn schedule_request_props(&mut self, dbus: &mut DBus) {
        self.all_props_request.reset();
        self.all_props_request = Oneshot::new(GetAllPropsOneshot);
        self.all_props_request
            .start(dbus, self.service.name().to_string());
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        self.new_icon_subscription = Oneshot::new(NewIconSubscription);
        self.new_icon_subscription
            .start(dbus, self.service.name().to_string());
        self.all_props_request
            .start(dbus, self.service.name().to_string());
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
        let mut get_layout = Oneshot::new(GetLayout::new(self.service.name()));
        get_layout.start(dbus, (self.service.name().to_string(), self.menu.clone()));
        self.get_layout = get_layout;
    }

    fn on_menu_received(&mut self, menu: String, dbus: &mut DBus) {
        if !self.menu.is_empty() {
            return;
        }

        self.menu = menu;
        self.schedule_get_layout(dbus);
        self.layout_updated_subscription
            .start(dbus, (self.service.name().to_string(), self.menu.clone()));
        self.items_properties_updated_subscription
            .start(dbus, (self.service.name().to_string(), self.menu.clone()));
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
            log::info!("Received requested props for {:?}", self.service);

            self.on_menu_received(menu, dbus);
            return self.on_icon_received(icon);
        }

        if let Some(()) = self.new_icon_subscription.process(message) {
            log::info!("Subscribed to NewIcon");
            return None;
        }

        if let Some(AllPropsUpdate { icon, .. }) = self.all_props_subscription.process(message) {
            log::info!("Received updated props for {:?}", self.service);
            if let Some(icon) = icon {
                return self.on_icon_received(icon);
            }
        }

        if let Some(layout) = self.get_layout.process(message) {
            log::info!("Got layout");
            return self.on_layout_receieved(layout);
        }

        if let Some(_) = self.layout_updated_subscription.process(message) {
            log::info!("Subscribed to LayoutUpdated");
            return None;
        }

        if let Some(_) = self.items_properties_updated_subscription.process(message) {
            log::info!("Subscribed to ItemPropertiesUpdated");
            return None;
        }

        if parse_new_icon_signal(message, self.service.raw_address()).is_ok() {
            log::info!("Received NewIcon signal");
            self.schedule_request_props(dbus);
            return None;
        }

        if parse_layout_updated_signal(message, self.service.raw_address(), &self.menu).is_ok() {
            log::info!("Received LayoutUpdated signal");
            self.schedule_get_layout(dbus);
            return None;
        }
        if parse_items_properties_updated_signal(message, self.service.raw_address(), &self.menu)
            .is_ok()
        {
            log::info!("Received ItemsPropertiesUpdated signal");
            self.schedule_get_layout(dbus);
            return None;
        }

        None
    }
}

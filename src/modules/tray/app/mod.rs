use crate::{
    dbus::{DBus, Message, Oneshot, Subscription},
    modules::{TrayIcon, TrayItem},
};
use get_layout::GetLayout;
use ksni::{
    AllProps, AllPropsSubscription, AllPropsUpdate, GetAllPropsOneshot, NewIconSubscription,
};

mod get_layout;
mod ksni;

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

use crate::{
    modules::{SessionDBus, TrayIcon, TrayIconPixmap, TrayItem, tray::service::Service},
    utils::{
        StringRef, StringRefExt as _, dbus::infallible_property::InfalliblePropertyGetAndSubscribe,
    },
};
use anyhow::Result;
use dbus::{
    EncodeError, IncomingMessage,
    messages::sni_host::{
        Event, EventArgs, IconName, IconPixmap, IconPixmapBytes, ItemsPropertiesUpdatedSignal,
        ItemsPropertiesUpdatedSubscribe, ItemsPropertiesUpdatedUnsubscribe, LayoutUpdatedSignal,
        LayoutUpdatedSubscribe, LayoutUpdatedUnsubscribe, Menu, NewIconSignal, NewIconSubscribe,
        NewIconUnsubscribe,
    },
    messaging::{DBusEncode, reply_handler::ReplyHandler},
};

mod state;
use state::State;

mod get_layout;
use get_layout::{GetLayout, VecOfTrayItems};

pub(crate) struct App {
    service: Service,

    menu_prop: InfalliblePropertyGetAndSubscribe<Menu<StringRef>>,
    icon_name_prop: InfalliblePropertyGetAndSubscribe<IconName<StringRef>>,
    icon_pixmap_prop: InfalliblePropertyGetAndSubscribe<IconPixmap<StringRef, VecOfU8>>,

    get_layout: Option<ReplyHandler<GetLayout>>,

    menu: StringRef,
    state: State,
}

struct VecOfU8(Vec<u8>);
impl IconPixmapBytes for VecOfU8 {
    fn new() -> Self {
        Self(vec![])
    }

    fn push(&mut self, v: u8) {
        self.0.push(v);
    }
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

            menu_prop: InfalliblePropertyGetAndSubscribe::new(),
            icon_name_prop: InfalliblePropertyGetAndSubscribe::new(),
            icon_pixmap_prop: InfalliblePropertyGetAndSubscribe::new(),

            get_layout: None,

            menu: StringRef::new(""),
            state: State::Nothing,
        }
    }

    fn schedule_request_props(&mut self) {
        self.menu_prop
            .get(Menu::new(self.service.name()), SessionDBus::queue());
        self.icon_name_prop
            .get(IconName::new(self.service.name()), SessionDBus::queue());
        self.icon_pixmap_prop
            .get(IconPixmap::new(self.service.name()), SessionDBus::queue());
    }

    pub(crate) fn init(&mut self) -> Result<()> {
        let mut bytes = [0; 1_024];
        let buf = NewIconSubscribe::encode(self.service.name_str(), &mut bytes)?;
        SessionDBus::queue().push_raw(buf);

        self.menu_prop
            .get_and_subscribe(Menu::new(self.service.name()), SessionDBus::queue());
        self.icon_name_prop
            .get_and_subscribe(IconName::new(self.service.name()), SessionDBus::queue());
        self.icon_pixmap_prop
            .get_and_subscribe(IconPixmap::new(self.service.name()), SessionDBus::queue());

        Ok(())
    }

    pub(crate) fn reset(&mut self) -> Result<()> {
        let mut bytes = [0; 1_024];

        let buf = NewIconUnsubscribe::encode(self.service.name_str(), &mut bytes)?;
        SessionDBus::queue().push_raw(buf);

        self.menu_prop.unsubscribe(SessionDBus::queue());
        self.icon_name_prop.unsubscribe(SessionDBus::queue());
        self.icon_pixmap_prop.unsubscribe(SessionDBus::queue());

        let buf = LayoutUpdatedUnsubscribe::encode(
            (self.service.name_str(), self.menu.as_str()),
            &mut bytes,
        )?;
        SessionDBus::queue().push_raw(buf);

        let buf = ItemsPropertiesUpdatedUnsubscribe::encode(
            (self.service.name_str(), self.menu.as_str()),
            &mut bytes,
        )?;
        SessionDBus::queue().push_raw(buf);

        Ok(())
    }

    fn schedule_get_layout(&mut self) -> Result<()> {
        let mut buf = [0; 1_024];
        let buf = GetLayout::encode((self.service.name_str(), self.menu.as_str()), &mut buf)?;

        let message = GetLayout::new(self.service.name());
        let serial = SessionDBus::queue().push_raw(buf);

        self.get_layout = Some(ReplyHandler::new(serial, message));
        Ok(())
    }

    fn on_menu_received(&mut self, menu: &str) -> Result<()> {
        if self.menu != "" {
            return Ok(());
        }

        self.menu = StringRef::new(menu);
        let mut bytes = [0; 1_024];
        self.schedule_get_layout()?;

        let buf = LayoutUpdatedSubscribe::encode((self.service.name_str(), menu), &mut bytes)?;
        SessionDBus::queue().push_raw(buf);

        let buf =
            ItemsPropertiesUpdatedSubscribe::encode((self.service.name_str(), menu), &mut bytes)?;
        SessionDBus::queue().push_raw(buf);
        Ok(())
    }

    pub(crate) fn handle(&mut self, message: IncomingMessage<'_>) -> Result<Option<TrayEvent>> {
        if message.sender != Some(self.service.raw_address_str()) {
            return Ok(None);
        }

        if let Some(menu) = self
            .menu_prop
            .handle_reply_or_signal(message, SessionDBus::queue())
        {
            log::info!(target: "Tray", "Received menu {:?} - {menu:?}", self.service);
            self.on_menu_received(menu)?;
            Ok(None)
        } else if let Some(name_or_path) = self
            .icon_name_prop
            .handle_reply_or_signal(message, SessionDBus::queue())
        {
            if name_or_path.is_empty() {
                Ok(None)
            } else {
                let icon = TrayIcon::detect_name_or_path(name_or_path);
                log::info!(target: "Tray", "Received icon name {:?}", self.service);
                let event = self.state.on_icon_received(icon);
                Ok(event)
            }
        } else if let Some((width, height, VecOfU8(bytes))) = self
            .icon_pixmap_prop
            .handle_reply_or_signal(message, SessionDBus::queue())
        {
            log::info!(target: "Tray", "Received icon pixmap {:?}", self.service);
            let event = self
                .state
                .on_icon_received(TrayIcon::Pixmap(TrayIconPixmap {
                    width,
                    height,
                    bytes: bytes.into(),
                }));
            Ok(event)
        } else if let Some(get_layout) = &self.get_layout
            && let Some(VecOfTrayItems(layout)) = get_layout.handle(message)?
        {
            log::info!(target: "Tray", "Got layout");
            let event = self.state.on_layout_receieved(layout);
            Ok(event)
        } else if NewIconSignal::matches(message, self.service.raw_address_str()) {
            log::info!(target: "Tray", "Received NewIcon signal");
            self.schedule_request_props();
            Ok(None)
        } else if LayoutUpdatedSignal::matches(
            message,
            self.service.raw_address_str(),
            self.menu.as_str(),
        ) {
            log::info!(target: "Tray", "Received LayoutUpdated signal");
            self.schedule_get_layout()?;
            Ok(None)
        } else if ItemsPropertiesUpdatedSignal::matches(
            message,
            self.service.raw_address_str(),
            self.menu.as_str(),
        ) {
            log::info!(target: "Tray", "Received ItemsPropertiesUpdated signal");
            self.schedule_get_layout()?;
            Ok(None)
        } else {
            Ok(None)
        }
    }

    pub(crate) fn trigger(&self, id: i32) -> Result<(), EncodeError> {
        let timestamp =
            u32::try_from(chrono::Utc::now().timestamp()).map_err(|_| EncodeError::ValueTooLong)?;
        let args = EventArgs {
            id,
            timestamp,
            destination: self.service.name_str(),
            path: self.menu.as_str(),
        };
        let mut buf = [0; 1_024];
        let buf = Event::encode(args, &mut buf)?;
        SessionDBus::queue().push_raw(buf);

        Ok(())
    }
}

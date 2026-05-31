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
        Event, IconName, IconPixmap, IconPixmapBytes, ItemsPropertiesUpdatedSignal,
        LayoutUpdatedSignal, Menu, NewIconSignal,
    },
    messaging::reply_handler::ReplyHandler,
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

            menu_prop: InfalliblePropertyGetAndSubscribe::new(SessionDBus::queue()),
            icon_name_prop: InfalliblePropertyGetAndSubscribe::new(SessionDBus::queue()),
            icon_pixmap_prop: InfalliblePropertyGetAndSubscribe::new(SessionDBus::queue()),

            get_layout: None,

            menu: StringRef::new(""),
            state: State::Nothing,
        }
    }

    fn schedule_request_props(&mut self) {
        self.menu_prop.get(Menu::new(self.service.name()));
        self.icon_name_prop.get(IconName::new(self.service.name()));
        self.icon_pixmap_prop
            .get(IconPixmap::new(self.service.name()));
    }

    pub(crate) fn init(&mut self) -> Result<()> {
        NewIconSignal::subscribe(
            &mut [0; 1_024],
            SessionDBus::queue(),
            self.service.name_str(),
        )?;

        self.menu_prop
            .get_and_subscribe(Menu::new(self.service.name()));
        self.icon_name_prop
            .get_and_subscribe(IconName::new(self.service.name()));
        self.icon_pixmap_prop
            .get_and_subscribe(IconPixmap::new(self.service.name()));

        Ok(())
    }

    pub(crate) fn reset(&mut self) -> Result<()> {
        let mut buf = [0; 1_024];

        NewIconSignal::unsubscribe(&mut buf, SessionDBus::queue(), self.service.name_str())?;

        self.menu_prop.unsubscribe();
        self.icon_name_prop.unsubscribe();
        self.icon_pixmap_prop.unsubscribe();

        LayoutUpdatedSignal::unsubscribe(
            &mut buf,
            SessionDBus::queue(),
            self.service.name_str(),
            self.menu.as_str(),
        )?;
        ItemsPropertiesUpdatedSignal::unsubscribe(
            &mut buf,
            SessionDBus::queue(),
            self.service.name_str(),
            self.menu.as_str(),
        )?;
        Ok(())
    }

    fn schedule_get_layout(&mut self) -> Result<()> {
        self.get_layout = Some(GetLayout::send(
            &mut [0; 1_024],
            SessionDBus::queue(),
            self.service.name(),
            self.menu.as_str(),
        )?);
        Ok(())
    }

    fn on_menu_received(&mut self, menu: &str) -> Result<()> {
        if self.menu != "" {
            return Ok(());
        }

        self.menu = StringRef::new(menu);
        let mut buf = [0; 1_024];
        self.schedule_get_layout()?;
        LayoutUpdatedSignal::subscribe(
            &mut buf,
            SessionDBus::queue(),
            self.service.name_str(),
            menu,
        )?;
        ItemsPropertiesUpdatedSignal::subscribe(
            &mut buf,
            SessionDBus::queue(),
            self.service.name_str(),
            menu,
        )?;
        Ok(())
    }

    pub(crate) fn handle(&mut self, message: IncomingMessage<'_>) -> Result<Option<TrayEvent>> {
        if let Some(menu) = self.menu_prop.handle_reply_or_signal(message) {
            log::info!(target: "Tray", "Received menu {:?} - {menu:?}", self.service);
            self.on_menu_received(menu)?;
            Ok(None)
        } else if let Some(name_or_path) = self.icon_name_prop.handle_reply_or_signal(message) {
            if name_or_path.is_empty() {
                Ok(None)
            } else {
                let icon = TrayIcon::detect_name_or_path(name_or_path);
                log::info!(target: "Tray", "Received icon name {:?}", self.service);
                let event = self.state.on_icon_received(icon);
                Ok(event)
            }
        } else if let Some((width, height, VecOfU8(bytes))) =
            self.icon_pixmap_prop.handle_reply_or_signal(message)
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

        Event::send(
            &mut [0; 1_024],
            SessionDBus::queue(),
            id,
            timestamp,
            self.service.name_str(),
            self.menu.as_str(),
        )?;

        Ok(())
    }
}

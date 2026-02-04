use crate::{
    dbus::{DBus, Message},
    liburing::IoUring,
    modules::network::{
        active_connection_type::ActiveConnectionType,
        primary_connection::{PrimaryConnection, PrimaryConnectionEvent},
    },
};
use anyhow::Result;

#[derive(Default)]
enum State {
    #[default]
    Disconnected,
    ConnectedAndHavePath,
    ConnectedAndHavePathAndType,
}

pub(crate) struct WirelessConnection {
    primary_connection: PrimaryConnection,
    active_connection_type: ActiveConnectionType,
    state: State,
}

pub(crate) enum WirelessConnectionEvent {
    Connected(String),
    Disconnected,
}

impl WirelessConnection {
    pub(crate) fn new() -> Self {
        Self {
            primary_connection: PrimaryConnection::new(),
            active_connection_type: ActiveConnectionType::new(),
            state: State::default(),
        }
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus, ring: &mut IoUring) -> Result<()> {
        self.primary_connection.init(dbus, ring)
    }

    fn on_primary_connection_event(
        &mut self,
        dbus: &mut DBus,
        e: PrimaryConnectionEvent,
        ring: &mut IoUring,
    ) -> Result<Option<WirelessConnectionEvent>> {
        match e {
            PrimaryConnectionEvent::Connected(path) => {
                self.active_connection_type.request(dbus, &path, ring)?;
                self.state = State::ConnectedAndHavePath;
                Ok(None)
            }
            PrimaryConnectionEvent::Disconnected => {
                self.active_connection_type.reset();
                self.state = State::Disconnected;
                Ok(Some(WirelessConnectionEvent::Disconnected))
            }
        }
    }

    fn on_active_connection_type_received(
        &mut self,
        is_wireless: bool,
        path: String,
    ) -> Option<WirelessConnectionEvent> {
        if is_wireless {
            self.state = State::ConnectedAndHavePathAndType;
            Some(WirelessConnectionEvent::Connected(path))
        } else {
            self.state = State::Disconnected;
            Some(WirelessConnectionEvent::Disconnected)
        }
    }

    pub(crate) fn on_message(
        &mut self,
        dbus: &mut DBus,
        message: &Message,
        ring: &mut IoUring,
    ) -> Result<Option<WirelessConnectionEvent>> {
        if let Some(e) = self.primary_connection.on_message(message) {
            return self.on_primary_connection_event(dbus, e, ring);
        }

        if let Some((is_wireless, path)) = self.active_connection_type.on_message(message) {
            return Ok(self.on_active_connection_type_received(is_wireless, path));
        }

        Ok(None)
    }
}

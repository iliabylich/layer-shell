use crate::{
    dbus::{DBus, Message},
    modules::network::{
        active_connection_type::ActiveConnectionType,
        primary_connection::{PrimaryConnection, PrimaryConnectionEvent},
    },
};

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

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        self.primary_connection.init(dbus);
    }

    fn on_primary_connection_event(
        &mut self,
        dbus: &mut DBus,
        e: PrimaryConnectionEvent,
    ) -> Option<WirelessConnectionEvent> {
        match e {
            PrimaryConnectionEvent::Connected(path) => {
                self.active_connection_type.request(dbus, &path);
                self.state = State::ConnectedAndHavePath;
                None
            }
            PrimaryConnectionEvent::Disconnected => {
                self.active_connection_type.reset();
                self.state = State::Disconnected;
                Some(WirelessConnectionEvent::Disconnected)
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
    ) -> Option<WirelessConnectionEvent> {
        if let Some(e) = self.primary_connection.on_message(message) {
            return self.on_primary_connection_event(dbus, e);
        }

        if let Some((is_wireless, path)) = self.active_connection_type.on_message(message) {
            return self.on_active_connection_type_received(is_wireless, path);
        }

        None
    }
}

use crate::{
    modules::network::{
        active_connection_type::ActiveConnectionType,
        primary_connection::{PrimaryConnection, PrimaryConnectionEvent},
    },
    utils::StringRef,
};
use dbus::IncomingMessage;

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

#[derive(Debug)]
pub(crate) enum WirelessConnectionEvent {
    Connected(StringRef),
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

    pub(crate) fn start(&mut self) {
        self.primary_connection.start();
    }

    fn on_primary_connection_event(
        &mut self,
        e: PrimaryConnectionEvent,
    ) -> Option<WirelessConnectionEvent> {
        match e {
            PrimaryConnectionEvent::Connected(path) => {
                self.active_connection_type.start(path);
                self.state = State::ConnectedAndHavePath;
                None
            }
            PrimaryConnectionEvent::Disconnected => {
                self.active_connection_type.stop();
                self.state = State::Disconnected;
                Some(WirelessConnectionEvent::Disconnected)
            }
        }
    }

    fn on_active_connection_type_received(
        &mut self,
        is_wireless: bool,
        path: StringRef,
    ) -> WirelessConnectionEvent {
        if is_wireless {
            self.state = State::ConnectedAndHavePathAndType;
            WirelessConnectionEvent::Connected(path)
        } else {
            self.state = State::Disconnected;
            WirelessConnectionEvent::Disconnected
        }
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<WirelessConnectionEvent> {
        if let Some(e) = self.primary_connection.handle(message) {
            return self.on_primary_connection_event(e);
        }

        if let Some((is_wireless, path)) = self.active_connection_type.handle(message) {
            return Some(self.on_active_connection_type_received(is_wireless, path));
        }

        None
    }
}

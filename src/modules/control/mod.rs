use crate::{
    channel::VerboseSender,
    dbus::{register_org_me_layer_shell_control, OrgMeLayerShellControl},
    Event,
};
use anyhow::{Context as _, Result};
use dbus::{
    blocking::Connection,
    channel::{BusType, Channel},
    MessageType,
};
use dbus_crossroads::Crossroads;
use std::time::Duration;

pub(crate) struct ConnectedControl {
    conn: Connection,
    cr: Crossroads,
}

impl ConnectedControl {
    fn try_new(tx: VerboseSender<Event>) -> Result<Self> {
        let mut channel =
            Channel::get_private(BusType::Session).context("failed to connect to DBus")?;
        channel.set_watch_enabled(true);
        let conn = Connection::from(channel);

        conn.request_name("org.me.LayerShellControl", true, true, true)?;

        let mut cr = Crossroads::new();
        let token = register_org_me_layer_shell_control::<DBusService>(&mut cr);
        cr.insert("/Control", &[token], DBusService { tx });

        Ok(Self { conn, cr })
    }

    fn read(&mut self) {
        while let Ok(Some(message)) = self
            .conn
            .channel()
            .blocking_pop_message(Duration::from_secs(0))
        {
            let dbg_message = format!("{:?}", message);
            if message.msg_type() == MessageType::MethodCall
                && self.cr.handle_message(message, &self.conn).is_err()
            {
                log::error!("failed to process {dbg_message}");
            }
        }
    }

    fn fd(&self) -> i32 {
        self.conn.channel().watch().fd
    }
}

pub(crate) enum Control {
    Connected(ConnectedControl),
    Disconnected,
}

impl Control {
    pub(crate) fn new(tx: VerboseSender<Event>) -> Self {
        ConnectedControl::try_new(tx)
            .map(Self::Connected)
            .inspect_err(|err| log::error!("{:?}", err))
            .unwrap_or(Self::Disconnected)
    }

    pub(crate) fn read(&mut self) {
        if let Self::Connected(control) = self {
            control.read();
        }
    }

    pub(crate) fn fd(&self) -> Option<i32> {
        if let Self::Connected(control) = self {
            Some(control.fd())
        } else {
            None
        }
    }
}

struct DBusService {
    tx: VerboseSender<Event>,
}

impl OrgMeLayerShellControl for DBusService {
    fn toggle_launcher(&mut self) -> std::result::Result<(), dbus::MethodErr> {
        self.tx.send(Event::ToggleLauncher);
        Ok(())
    }

    fn toggle_session_screen(&mut self) -> std::result::Result<(), dbus::MethodErr> {
        self.tx.send(Event::ToggleSessionScreen);
        Ok(())
    }

    fn exit(&mut self) -> std::result::Result<(), dbus::MethodErr> {
        std::process::exit(0);
    }
}

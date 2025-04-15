use crate::{
    Event,
    channel::EventSender,
    dbus::{OrgMeLayerShellControl, register_org_me_layer_shell_control},
    fd_id::FdId,
    modules::Module,
};
use anyhow::{Context as _, Result};
use dbus::{
    MessageType,
    blocking::Connection,
    channel::{BusType, Channel},
};
use dbus_crossroads::Crossroads;
use std::{
    os::fd::{AsRawFd, RawFd},
    time::Duration,
};

pub(crate) struct Control {
    conn: Connection,
    cr: Crossroads,
}

impl Module for Control {
    const FD_ID: FdId = FdId::ControlDBus;
    const NAME: &str = "Control";

    type ReadOutput = ();

    fn new(tx: &EventSender) -> Result<Self> {
        let mut channel =
            Channel::get_private(BusType::Session).context("failed to connect to DBus")?;
        channel.set_watch_enabled(true);
        let conn = Connection::from(channel);

        conn.request_name("org.me.LayerShellControl", true, true, true)?;

        let mut cr = Crossroads::new();
        let token = register_org_me_layer_shell_control::<DBusService>(&mut cr);
        cr.insert("/Control", &[token], DBusService { tx: tx.clone() });

        Ok(Self { conn, cr })
    }

    fn read_events(&mut self) -> Result<()> {
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
        Ok(())
    }
}

impl AsRawFd for Control {
    fn as_raw_fd(&self) -> RawFd {
        self.conn.channel().watch().fd
    }
}

struct DBusService {
    tx: EventSender,
}

impl OrgMeLayerShellControl for DBusService {
    fn toggle_launcher(&mut self) -> std::result::Result<(), dbus::MethodErr> {
        self.tx.send(Event::ToggleLauncher());
        Ok(())
    }

    fn toggle_session_screen(&mut self) -> std::result::Result<(), dbus::MethodErr> {
        self.tx.send(Event::ToggleSessionScreen());
        Ok(())
    }

    fn reload_styles(&mut self) -> std::result::Result<(), dbus::MethodErr> {
        self.tx.send(Event::ReloadStyles());
        Ok(())
    }

    fn exit(&mut self) -> std::result::Result<(), dbus::MethodErr> {
        std::process::exit(0);
    }
}

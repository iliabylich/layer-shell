use crate::{
    Event,
    channel::VerboseSender,
    dbus::{OrgMeLayerShellControl, register_org_me_layer_shell_control},
    epoll::{FdId, Reader},
    modules::maybe_connected::MaybeConnected,
};
use anyhow::{Context as _, Result};
use dbus::{
    MessageType,
    blocking::Connection,
    channel::{BusType, Channel},
};
use dbus_crossroads::Crossroads;
use std::{os::fd::RawFd, time::Duration};

pub(crate) struct Control {
    conn: Connection,
    cr: Crossroads,
}

impl Control {
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

    pub(crate) fn new(tx: VerboseSender<Event>) -> MaybeConnected<Self> {
        MaybeConnected::new(Self::try_new(tx))
    }
}

impl Reader for Control {
    type Output = ();

    const NAME: &str = "Control";

    fn read(&mut self) -> Result<Self::Output> {
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

    fn fd(&self) -> RawFd {
        self.conn.channel().watch().fd
    }

    fn fd_id(&self) -> FdId {
        FdId::ControlDBus
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

    fn reload_styles(&mut self) -> std::result::Result<(), dbus::MethodErr> {
        self.tx.send(Event::ReloadStyles);
        Ok(())
    }

    fn exit(&mut self) -> std::result::Result<(), dbus::MethodErr> {
        std::process::exit(0);
    }
}

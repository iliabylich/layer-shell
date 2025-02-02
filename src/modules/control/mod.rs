use crate::{
    dbus::{register_org_me_layer_shell_control, OrgMeLayerShellControl},
    scheduler::Actor,
    Event,
};
use anyhow::{Context as _, Result};
use dbus::{blocking::Connection, channel::MatchingReceiver as _};
use dbus_crossroads::Crossroads;
use std::{ops::ControlFlow, time::Duration};

pub(crate) struct Control {
    conn: Connection,
}

impl Actor for Control {
    fn name() -> &'static str {
        "Control"
    }

    fn start() -> Result<Box<dyn Actor>> {
        let conn = Connection::new_session().context("failed to connect to DBus")?;

        conn.request_name("org.me.LayerShellControl", true, true, true)?;

        let mut cr = Crossroads::new();
        let token = register_org_me_layer_shell_control::<DBusService>(&mut cr);

        cr.insert("/Control", &[token], DBusService);

        conn.start_receive(
            dbus::message::MatchRule::new_method_call(),
            Box::new(move |msg, conn| {
                if cr.handle_message(msg, conn).is_err() {
                    log::error!("Failed to handle message");
                }
                true
            }),
        );

        Ok(Box::new(Self { conn }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        while self.conn.process(Duration::from_millis(200))? {}
        Ok(ControlFlow::Continue(Duration::from_millis(200)))
    }

    fn exec(&mut self, _: &crate::Command) -> Result<ControlFlow<()>> {
        Ok(ControlFlow::Break(()))
    }
}

impl std::fmt::Debug for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Control").field("conn", &"<conn>").finish()
    }
}

struct DBusService;

impl OrgMeLayerShellControl for DBusService {
    fn toggle_launcher(&mut self) -> std::result::Result<(), dbus::MethodErr> {
        Event::ToggleLauncher.emit();
        Ok(())
    }

    fn toggle_session_screen(&mut self) -> std::result::Result<(), dbus::MethodErr> {
        Event::ToggleSessionScreen.emit();
        Ok(())
    }

    fn exit(&mut self) -> std::result::Result<(), dbus::MethodErr> {
        std::process::exit(0);
    }
}

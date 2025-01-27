use crate::{
    dbus::{register_org_me_layer_shell_control, OrgMeLayerShellControl},
    scheduler::{Module, RepeatingModule},
    Event,
};
use anyhow::{Context as _, Result};
use dbus::{blocking::Connection, channel::MatchingReceiver as _};
use dbus_crossroads::Crossroads;
use std::time::Duration;

pub(crate) struct Control;

impl Module for Control {
    const NAME: &str = "Control";

    fn start() -> Result<Option<Box<dyn RepeatingModule>>> {
        let conn = Connection::new_session().context("failed to connect to DBus")?;

        conn.request_name("org.me.LayerShellControl", true, true, true)?;

        let mut cr = Crossroads::new();
        let token = register_org_me_layer_shell_control::<Control>(&mut cr);

        cr.insert("/Control", &[token], Control);

        conn.start_receive(
            dbus::message::MatchRule::new_method_call(),
            Box::new(move |msg, conn| {
                if cr.handle_message(msg, conn).is_err() {
                    log::error!("Failed to handle message");
                }
                true
            }),
        );

        Ok(Some(Box::new(ControlTick { conn })))
    }
}

struct ControlTick {
    conn: Connection,
}

impl RepeatingModule for ControlTick {
    fn tick(&mut self) -> Result<Duration> {
        while self.conn.process(Duration::from_millis(200))? {}

        Ok(Duration::from_millis(200))
    }
}

impl OrgMeLayerShellControl for Control {
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

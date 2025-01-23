use crate::{
    dbus::{register_org_me_layer_shell_control, OrgMeLayerShellControl},
    Event,
};
use anyhow::{Context as _, Result};
use dbus::{blocking::Connection, channel::MatchingReceiver as _};
use dbus_crossroads::Crossroads;
use std::time::Duration;

pub(crate) fn setup() {
    std::thread::spawn(|| {
        if let Err(err) = try_setup() {
            log::error!("Failed to spawn session thread: {:?}", err);
        }
    });
}

struct Control;

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

pub(crate) fn try_setup() -> Result<()> {
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

    loop {
        conn.process(Duration::from_millis(200))?;
    }
}

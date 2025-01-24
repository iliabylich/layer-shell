use crate::{
    dbus::{register_org_me_layer_shell_control, OrgMeLayerShellControl},
    scheduler::Module,
    Event,
};
use anyhow::{Context as _, Result};
use dbus::{blocking::Connection, channel::MatchingReceiver as _};
use dbus_crossroads::Crossroads;
use std::{any::Any, time::Duration};

pub(crate) struct Control;

impl Module for Control {
    const NAME: &str = "Control";
    const INTERVAL: Option<u64> = Some(200);

    fn start() -> Result<Box<dyn Any + Send + 'static>> {
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

        Ok(Box::new(conn))
    }

    fn tick(state: &mut Box<dyn Any + Send + 'static>) -> Result<()> {
        let conn = state
            .downcast_ref::<Connection>()
            .context("Control state is malformed")?;
        conn.process(Duration::from_millis(200))?;
        Ok(())
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

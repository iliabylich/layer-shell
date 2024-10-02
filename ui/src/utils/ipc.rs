use anyhow::{Context, Result};

use crate::{
    layers::{Launcher, LogoutScreen},
    utils::LayerWindow,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum IPCMessage {
    Stop,
    ToggleLauncher,
    ToggleLogoutScreen,
}

impl IPCMessage {
    fn execute(self) {
        match self {
            IPCMessage::Stop => std::process::exit(0),
            IPCMessage::ToggleLauncher => Launcher::toggle(),
            IPCMessage::ToggleLogoutScreen => LogoutScreen::toggle(),
        }
    }
}

pub(crate) struct IPC {}

impl IPC {
    pub(crate) fn spawn() -> Result<()> {
        let config = Config::new()?;
        config.write_pid()?;
        gtk4::glib::unix_signal_add(10 /* USR1 */, move || {
            if let Some(message) = config.read_message() {
                message.execute();
            }
            gtk4::glib::ControlFlow::Continue
        });
        Ok(())
    }

    pub(crate) fn send_to_running_instance(message: IPCMessage) -> Result<()> {
        let config = Config::new()?;
        if let Some(pid) = config.read_pid() {
            config.write_message(message);
            std::process::Command::new("kill")
                .args(["-USR1", pid.as_str()])
                .spawn()
                .context("failed to send USR1 to running instance")?;
        }
        Ok(())
    }
}

struct Config {
    pipe: String,
    pidfile: String,
}

impl Config {
    fn new() -> Result<Self> {
        let dir = format!(
            "{}/.config/layer-shell",
            std::env::var("HOME").context("no $HOME")?
        );
        std::fs::create_dir_all(&dir).context("failed to create dir for shared message file")?;

        let pipe = format!("{}/.message", dir);
        std::fs::File::create(&pipe).context("failed to create shared message file")?;

        let pidfile = format!("{}/.pid", dir);

        Ok(Self { pipe, pidfile })
    }

    fn write_pid(&self) -> Result<()> {
        std::fs::write(&self.pidfile, format!("{}", std::process::id()))
            .context("failed to write PID to pidfile")
    }

    fn read_pid(&self) -> Option<String> {
        std::fs::read_to_string(&self.pidfile).ok()
    }

    fn write_message(&self, message: IPCMessage) {
        let message = serde_json::to_string(&message).expect("failed to serialize IPCMessage");
        std::fs::write(&self.pipe, message).unwrap();
    }

    fn read_message(&self) -> Option<IPCMessage> {
        let command = std::fs::read_to_string(&self.pipe).ok()?;
        serde_json::from_str::<IPCMessage>(&command).ok()
    }
}

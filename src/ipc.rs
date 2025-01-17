use std::sync::LazyLock;

use crate::{fatal::fatal, Event};
use anyhow::{Context, Result};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum IPCMessage {
    Exit,
    ToggleLauncher,
    ToggleSessionScreen,
}

impl IPCMessage {
    fn execute(self) {
        match self {
            IPCMessage::Exit => std::process::exit(0),
            IPCMessage::ToggleLauncher => Event::ToggleLauncher.emit(),
            IPCMessage::ToggleSessionScreen => Event::ToggleSessionScreen.emit(),
        }
    }
}

pub(crate) struct IPC;

impl IPC {
    pub(crate) fn set_current_process_as_main() -> Result<()> {
        Config::write_pid()
    }

    pub(crate) fn send_to_running_instance(message: IPCMessage) -> Result<()> {
        if let Some(pid) = Config::read_pid() {
            Config::write_message(message);
            std::process::Command::new("kill")
                .args(["-USR1", pid.as_str()])
                .spawn()
                .context("failed to send USR1 to running instance")?;
        }
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn layer_shell_io_on_sigusr1() {
    if let Some(message) = Config::read_message() {
        message.execute();
    }
}

struct Config {
    pipe: String,
    pidfile: String,
}
static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let home = std::env::var("HOME").unwrap_or_else(|_| fatal!("no $HOME"));

    let dir = format!("{home}/.config/layer-shell",);
    if let Err(err) = std::fs::create_dir_all(&dir) {
        fatal!("failed to create dir for shared message file: {:?}", err);
    }

    let pipe = format!("{}/.message", dir);
    if let Err(err) = std::fs::File::create(&pipe) {
        fatal!("failed to create shared message file: {:?}", err);
    }

    let pidfile = format!("{}/.pid", dir);

    Config { pipe, pidfile }
});

impl Config {
    fn write_pid() -> Result<()> {
        std::fs::write(&CONFIG.pidfile, format!("{}", std::process::id()))
            .context("failed to write PID to pidfile")
    }

    fn read_pid() -> Option<String> {
        std::fs::read_to_string(&CONFIG.pidfile).ok()
    }

    fn write_message(message: IPCMessage) {
        let message = serde_json::to_string(&message)
            .unwrap_or_else(|err| fatal!("failed to serialize IPCMessage: {:?}", err));

        if let Err(err) = std::fs::write(&CONFIG.pipe, message) {
            fatal!("failed to write message: {:?}", err);
        }
    }

    fn read_message() -> Option<IPCMessage> {
        let command = std::fs::read_to_string(&CONFIG.pipe).ok()?;
        serde_json::from_str::<IPCMessage>(&command).ok()
    }
}

use std::sync::OnceLock;

use crate::{fatal::fatal, Event};
use anyhow::{bail, Context, Result};

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
    pub(crate) fn prepare() -> Result<()> {
        Config::init()
    }

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
static CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    fn init() -> Result<()> {
        let dir = format!(
            "{}/.config/layer-shell",
            std::env::var("HOME").context("no $HOME")?
        );
        std::fs::create_dir_all(&dir).context("failed to create dir for shared message file")?;

        let pipe = format!("{}/.message", dir);
        std::fs::File::create(&pipe).context("failed to create shared message file")?;

        let pidfile = format!("{}/.pid", dir);

        if CONFIG.set(Self { pipe, pidfile }).is_err() {
            bail!("Config has been already set");
        }

        Ok(())
    }

    fn get() -> &'static Self {
        match CONFIG.get() {
            Some(config) => config,
            None => {
                fatal!("Config has not been initialized");
            }
        }
    }

    fn write_pid() -> Result<()> {
        std::fs::write(&Self::get().pidfile, format!("{}", std::process::id()))
            .context("failed to write PID to pidfile")
    }

    fn read_pid() -> Option<String> {
        std::fs::read_to_string(&Self::get().pidfile).ok()
    }

    fn write_message(message: IPCMessage) {
        match serde_json::to_string(&message) {
            Ok(message) => {
                if let Err(err) = std::fs::write(&Self::get().pipe, message) {
                    fatal!("failed to write message: {:?}", err);
                }
            }
            Err(err) => {
                fatal!("failed to serialize IPCMessage: {:?}", err);
            }
        }
    }

    fn read_message() -> Option<IPCMessage> {
        let command = std::fs::read_to_string(&Self::get().pipe).ok()?;
        serde_json::from_str::<IPCMessage>(&command).ok()
    }
}

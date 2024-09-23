use crate::{
    layers::{Launcher, LogoutScreen},
    utils::ToggleWindow,
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
    pub(crate) fn subscribe() {
        let config = Config::new();
        config.write_pid();
        gtk4::glib::unix_signal_add(10 /* USR1 */, move || {
            if let Some(message) = config.read_message() {
                message.execute();
            }
            gtk4::glib::ControlFlow::Continue
        });
    }

    pub(crate) fn send(message: IPCMessage) {
        let config = Config::new();
        if let Some(pid) = config.read_pid() {
            config.write_message(message);
            std::process::Command::new("kill")
                .args(["-USR1", pid.as_str()])
                .spawn()
                .unwrap();
        }
    }
}

struct Config {
    pipe: String,
    pidfile: String,
}

impl Config {
    fn new() -> Self {
        let dir = format!("{}/.config/layer-shell", std::env::var("HOME").unwrap());
        std::fs::create_dir_all(&dir).unwrap();

        let pipe = format!("{}/.message", dir);
        std::fs::File::create(&pipe).unwrap();

        let pidfile = format!("{}/.pid", dir);

        Self { pipe, pidfile }
    }

    fn write_pid(&self) {
        std::fs::write(&self.pidfile, format!("{}", std::process::id())).unwrap();
    }

    fn read_pid(&self) -> Option<String> {
        std::fs::read_to_string(&self.pidfile).ok()
    }

    fn write_message(&self, message: IPCMessage) {
        let message = serde_json::to_string(&message).unwrap();
        std::fs::write(&self.pipe, message).unwrap();
    }

    fn read_message(&self) -> Option<IPCMessage> {
        let command = std::fs::read_to_string(&self.pipe).ok()?;
        serde_json::from_str::<IPCMessage>(&command).ok()
    }
}

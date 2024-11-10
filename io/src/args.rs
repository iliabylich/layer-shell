use crate::ipc::{IPCMessage, IPC};
use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};

#[derive(Parser, Debug, PartialEq)]
enum Args {
    Generate { shell: Shell },
    Start,
    Exit,
    Toggle { window_name: WindowName },
}

#[derive(Debug, PartialEq, Clone, clap::ValueEnum)]
enum WindowName {
    Launcher,
    LogoutScreen,
}

pub(crate) fn parse_args() -> Result<()> {
    let args = Args::parse();

    match args {
        Args::Start => {
            // The only case in which we proceeed to booting app
            return Ok(());
        }
        Args::Generate { shell } => {
            let mut cmd = Args::command();
            generate(shell, &mut cmd, "layer-shell", &mut std::io::stdout());
        }
        Args::Exit => {
            IPC::send_to_running_instance(IPCMessage::Exit)?;
        }
        Args::Toggle {
            window_name: WindowName::Launcher,
        } => {
            IPC::send_to_running_instance(IPCMessage::ToggleLauncher)?;
        }
        Args::Toggle {
            window_name: WindowName::LogoutScreen,
        } => {
            IPC::send_to_running_instance(IPCMessage::ToggleLogoutScreen)?;
        }
    }
    std::process::exit(0);
}

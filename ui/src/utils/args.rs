use crate::utils::{IPCMessage, IPC};
use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};

#[derive(Parser, Debug, PartialEq)]
enum Args {
    Generate { shell: Shell },
    Start,
    Stop,
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
        Args::Generate { shell } => {
            let mut cmd = Args::command();
            let bin_name = cmd.get_name().to_string();
            generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
            std::process::exit(0);
        }
        Args::Start => {
            // The only case in which we proceeed
            Ok(())
        }
        Args::Stop => {
            IPC::send_to_running_instance(IPCMessage::Stop)?;
            std::process::exit(1);
        }
        Args::Toggle {
            window_name: WindowName::Launcher,
        } => {
            IPC::send_to_running_instance(IPCMessage::ToggleLauncher)?;
            std::process::exit(1);
        }
        Args::Toggle {
            window_name: WindowName::LogoutScreen,
        } => {
            IPC::send_to_running_instance(IPCMessage::ToggleLogoutScreen)?;
            std::process::exit(1);
        }
    }
}

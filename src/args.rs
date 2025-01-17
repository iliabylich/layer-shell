use crate::{
    fatal::fatal,
    ipc::{IPCMessage, IPC},
};
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
    SessionScreen,
}

pub(crate) fn parse_args() {
    if let Err(err) = try_parse_args() {
        fatal!("Error while parsing args: {:?}", err);
    }
}

fn try_parse_args() -> Result<()> {
    let args = Args::parse();

    match args {
        Args::Start => {
            // The only case in which we proceeed to booting app
            Ok(())
        }
        Args::Generate { shell } => {
            let mut cmd = Args::command();
            generate(shell, &mut cmd, "layer-shell", &mut std::io::stdout());
            std::process::exit(0);
        }
        Args::Exit => {
            IPC::send_to_running_instance(IPCMessage::Exit)?;
            std::process::exit(0);
        }
        Args::Toggle {
            window_name: WindowName::Launcher,
        } => {
            IPC::send_to_running_instance(IPCMessage::ToggleLauncher)?;
            std::process::exit(0);
        }
        Args::Toggle {
            window_name: WindowName::SessionScreen,
        } => {
            IPC::send_to_running_instance(IPCMessage::ToggleSessionScreen)?;
            std::process::exit(0);
        }
    }
}

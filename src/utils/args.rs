use crate::utils::{DBus, DBusMessage};
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

pub(crate) fn parse_args() {
    let args = Args::parse();

    match args {
        Args::Generate { shell } => {
            let mut cmd = Args::command();
            eprintln!("Generating completion file for {shell:?}");
            let bin_name = cmd.get_name().to_string();
            generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
            std::process::exit(0);
        }
        Args::Start => {
            // The only case in which we proceeed
        }
        Args::Stop => {
            DBus::send(DBusMessage::Stop);
            std::process::exit(1);
        }
        Args::Toggle {
            window_name: WindowName::Launcher,
        } => {
            DBus::send(DBusMessage::ToggleLauncher);
            std::process::exit(1);
        }
        Args::Toggle {
            window_name: WindowName::LogoutScreen,
        } => {
            DBus::send(DBusMessage::ToggleLogoutScreen);
            std::process::exit(1);
        }
    }
}

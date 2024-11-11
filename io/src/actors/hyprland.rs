use crate::{Command, Event};
use anyhow::{bail, Context, Result};
use std::{collections::HashSet, sync::mpsc::Sender};
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    net::UnixStream,
};

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("Hyprland model error: {}\n{}", err, err.backtrace());
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    let mut lines = connect_to_hyprland().await?;

    let mut workspaces = Workspaces::new()
        .await
        .context("failed to get initial workspaces")?;
    tx.send(Event::from(&workspaces))
        .context("failed to send event")?;

    let lang = get_language()
        .await
        .context("failed to get initial language")?;
    tx.send(Event::Language(lang))
        .context("failed to send event")?;

    while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(hyprland_event) = HyprlandEvent::try_from(line) {
            match hyprland_event {
                HyprlandEvent::CreateWorkspace(_)
                | HyprlandEvent::DestroyWorkspace(_)
                | HyprlandEvent::Workspace(_) => {
                    workspaces.apply(hyprland_event);
                    tx.send(Event::from(&workspaces))
                        .context("failed to send event")?;
                }
                HyprlandEvent::LanguageChanged(lang) => {
                    tx.send(Event::Language(lang))
                        .context("failed to send event")?;
                }
                HyprlandEvent::Other => {}
            }
        }
    }

    Ok(())
}

async fn connect_to_hyprland() -> Result<Lines<BufReader<UnixStream>>> {
    let socket_path = format!(
        "{}/hypr/{}/.socket2.sock",
        std::env::var("XDG_RUNTIME_DIR").context("no XDG_RUNTIME_DIR variable")?,
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .context("no HYPRLAND_INSTANCE_SIGNATURE, are you in Hyprland?")?,
    );

    let socket = UnixStream::connect(&socket_path)
        .await
        .context("failed to open unix socket")?;
    let buffered = BufReader::new(socket);
    let lines = buffered.lines();
    Ok(lines)
}

#[derive(Clone, Debug)]
pub(crate) enum HyprlandEvent {
    CreateWorkspace(usize),
    DestroyWorkspace(usize),
    Workspace(usize),
    LanguageChanged(String),
    Other,
}

impl TryFrom<String> for HyprlandEvent {
    type Error = anyhow::Error;

    fn try_from(line: String) -> Result<Self> {
        let (event, payload) = line.split_once(">>").context("expected >> separator")?;

        let payload_as_usize = || {
            payload
                .parse::<usize>()
                .with_context(|| format!("non integer payload of event {event}: {payload}"))
        };

        match event {
            "createworkspace" => Ok(Self::CreateWorkspace(payload_as_usize()?)),
            "destroyworkspace" => Ok(Self::DestroyWorkspace(payload_as_usize()?)),
            "workspace" => Ok(Self::Workspace(payload_as_usize()?)),
            "activelayout" => match payload.split(",").last() {
                Some(lang) => Ok(Self::LanguageChanged(lang.to_string())),
                None => {
                    bail!("unexpected payload of activelayout: {payload}")
                }
            },
            _ => Ok(Self::Other),
        }
    }
}

struct Workspaces {
    workspace_ids: HashSet<usize>,
    active_id: usize,
}
impl Workspaces {
    async fn new() -> Result<Self> {
        #[derive(serde::Deserialize)]
        pub(crate) struct Workspace {
            pub(crate) id: usize,
        }
        let stdout = exec_hyprctl("workspaces").await?;
        let workspaces: Vec<Workspace> =
            serde_json::from_str(&stdout).context("invalid response from hyprctl workspaces -j")?;

        let stdout = exec_hyprctl("activeworkspace").await?;
        let active_workspace: Workspace = serde_json::from_str(&stdout)
            .context("invalid response from hyprctl activeworkspace -j")?;

        Ok(Self {
            workspace_ids: HashSet::from_iter(workspaces.into_iter().map(|w| w.id)),
            active_id: active_workspace.id,
        })
    }

    fn apply(&mut self, event: HyprlandEvent) {
        match event {
            HyprlandEvent::CreateWorkspace(idx) => {
                self.workspace_ids.insert(idx);
            }
            HyprlandEvent::DestroyWorkspace(idx) => {
                self.workspace_ids.remove(&idx);
            }
            HyprlandEvent::Workspace(idx) => {
                self.active_id = idx;
            }
            _ => {}
        }
    }
}
impl From<&Workspaces> for Event {
    fn from(workspaces: &Workspaces) -> Self {
        Event::Workspaces {
            ids: workspaces.workspace_ids.clone(),
            active_id: workspaces.active_id,
        }
    }
}

async fn get_language() -> Result<String> {
    #[derive(serde::Deserialize)]
    pub(crate) struct Devices {
        pub(crate) keyboards: Vec<Keyboard>,
    }
    #[derive(serde::Deserialize)]
    pub(crate) struct Keyboard {
        pub(crate) main: bool,
        pub(crate) active_keymap: String,
    }

    let stdout = exec_hyprctl("devices").await?;
    let devices: Devices =
        serde_json::from_str(&stdout).context("invalid response from hyprctl devices -j")?;

    let main_keyboard = devices
        .keyboards
        .into_iter()
        .find(|keyboard| keyboard.main)
        .context("expected at least one hyprland device")?;

    Ok(main_keyboard.active_keymap)
}

async fn exec_hyprctl(command: &str) -> Result<String> {
    let stdout = tokio::process::Command::new("hyprctl")
        .args([command, "-j"])
        .output()
        .await
        .with_context(|| format!("failed to spawn hyprctl {command} -j"))?
        .stdout;
    String::from_utf8(stdout)
        .with_context(|| format!("hyprctl {command} -j returned non-utf-8 stdout"))
}

pub(crate) async fn on_command(command: &Command) {
    if let Command::GoToWorkspace(workspace_idx) = command {
        match tokio::process::Command::new("hyprctl")
            .args(["dispatch", "workspace", &format!("{}", workspace_idx + 1)])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(mut child) => {
                if let Err(err) = child.wait().await {
                    log::error!("Failed to spawn hyprctl: {}", err);
                }
            }
            Err(err) => {
                log::error!("Failed to spawn hyprctl: {}", err);
            }
        }
    }
}

use crate::{raw_event::RawEvent, Event};
use anyhow::{Context as _, Result};
use std::collections::HashSet;

pub(crate) struct State {
    workspace_ids: HashSet<usize>,
    active_workspace_id: usize,
    language: String,
}
impl State {
    pub(crate) async fn new() -> Result<Self> {
        let workspaces = get_workspaces().await?;
        let active_workspace = get_active_workspace().await?;
        let language = get_language().await?;

        Ok(Self {
            workspace_ids: HashSet::from_iter(workspaces.into_iter().map(|w| w.id)),
            active_workspace_id: active_workspace.id,
            language,
        })
    }

    pub(crate) fn apply(&mut self, event: RawEvent) -> Event {
        match event {
            RawEvent::CreateWorkspace(idx) => {
                self.workspace_ids.insert(idx);
                self.as_workspaces_changed_event()
            }
            RawEvent::DestroyWorkspace(idx) => {
                self.workspace_ids.remove(&idx);
                self.as_workspaces_changed_event()
            }
            RawEvent::Workspace(idx) => {
                self.active_workspace_id = idx;
                self.as_workspaces_changed_event()
            }
            RawEvent::LanguageChanged(language) => {
                self.language = language;
                self.as_language_changed_event()
            }
        }
    }

    pub(crate) fn as_workspaces_changed_event(&self) -> Event {
        Event::WorkspacesChanged {
            ids: self.workspace_ids.clone(),
            active_id: self.active_workspace_id,
        }
    }

    pub(crate) fn as_language_changed_event(&self) -> Event {
        Event::LanguageChanged(self.language.clone())
    }
}

#[derive(serde::Deserialize)]
struct Devices {
    keyboards: Vec<Keyboard>,
}
#[derive(serde::Deserialize)]
struct Keyboard {
    main: bool,
    active_keymap: String,
}
#[derive(serde::Deserialize)]
struct Workspace {
    id: usize,
}

async fn get_workspaces() -> Result<Vec<Workspace>> {
    let stdout = exec_hyprctl("workspaces").await?;
    serde_json::from_str(&stdout).context("invalid response from hyprctl workspaces -j")
}

async fn get_active_workspace() -> Result<Workspace> {
    let stdout = exec_hyprctl("activeworkspace").await?;
    serde_json::from_str(&stdout).context("invalid response from hyprctl activeworkspace -j")
}

async fn get_language() -> Result<String> {
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

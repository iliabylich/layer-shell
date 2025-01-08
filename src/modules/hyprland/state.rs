use crate::{modules::hyprland::raw_event::RawEvent, Event};
use anyhow::{Context as _, Result};
use std::collections::HashSet;

pub(crate) struct State {
    workspace_ids: HashSet<usize>,
    active_workspace_id: usize,
    language: String,
}
impl State {
    pub(crate) fn new() -> Result<Self> {
        let workspaces = get_workspaces()?;
        let active_workspace = get_active_workspace()?;
        let language = get_language()?;

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
        Event::Workspaces {
            ids: self
                .workspace_ids
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .into(),
            active_id: self.active_workspace_id,
        }
    }

    pub(crate) fn as_language_changed_event(&self) -> Event {
        Event::Language {
            lang: self.language.clone().into(),
        }
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

fn get_workspaces() -> Result<Vec<Workspace>> {
    let stdout = exec_hyprctl("workspaces")?;
    serde_json::from_str(&stdout).context("invalid response from hyprctl workspaces -j")
}

fn get_active_workspace() -> Result<Workspace> {
    let stdout = exec_hyprctl("activeworkspace")?;
    serde_json::from_str(&stdout).context("invalid response from hyprctl activeworkspace -j")
}

fn get_language() -> Result<String> {
    let stdout = exec_hyprctl("devices")?;
    let devices: Devices =
        serde_json::from_str(&stdout).context("invalid response from hyprctl devices -j")?;

    let main_keyboard = devices
        .keyboards
        .into_iter()
        .find(|keyboard| keyboard.main)
        .context("expected at least one hyprland device")?;

    Ok(main_keyboard.active_keymap)
}

fn exec_hyprctl(command: &str) -> Result<String> {
    let stdout = std::process::Command::new("hyprctl")
        .args([command, "-j"])
        .output()
        .with_context(|| format!("failed to spawn hyprctl {command} -j"))?
        .stdout;
    String::from_utf8(stdout)
        .with_context(|| format!("hyprctl {command} -j returned non-utf-8 stdout"))
}

use crate::{event::AppIcon, hyprctl};
use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    io::{BufRead as _, BufReader},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub(crate) struct SystemApp {
    pub(crate) name: String,
    pub(crate) exec: String,
    pub(crate) icon: AppIcon,
}

impl SystemApp {
    pub(crate) fn parse_all() -> Result<Vec<SystemApp>> {
        let filepaths = desktop_files()?;

        let mut apps = HashMap::new();

        for path in filepaths {
            match SystemApp::parse(&path) {
                Ok(desktop_entry) => {
                    apps.insert(desktop_entry.name.clone(), desktop_entry);
                }
                Err(err) => {
                    log::warn!("Failed to parse {:?}: {:?}", path, err);
                }
            }
        }

        let mut apps = apps.into_values().collect::<Vec<_>>();
        apps.sort_unstable_by(|app1, app2| app1.name.cmp(&app2.name));

        Ok(apps)
    }

    fn parse(path: &PathBuf) -> Result<SystemApp> {
        let f = std::fs::File::open(path).with_context(|| format!("failed to open {path:?}"))?;

        let mut lines = BufReader::new(f).lines();
        let mut in_desktop_entry_section = false;
        let mut name = None;
        let mut exec = None;
        let mut icon = None;
        while let Some(Ok(line)) = lines.next() {
            if line == "[Desktop Entry]" {
                in_desktop_entry_section = true
            } else if in_desktop_entry_section {
                if line.is_empty() {
                    in_desktop_entry_section = false;
                } else if let Some(rest) = line.strip_prefix("Name=") {
                    name = Some(rest.to_string());
                } else if let Some(rest) = line.strip_prefix("Exec=") {
                    exec = Some(rest.to_string());
                } else if let Some(rest) = line.strip_prefix("Icon=") {
                    icon = Some(rest.to_string());
                }
            }
        }

        let name = name.context("failed to get Name")?;
        let exec = exec
            .context("failed to get Exec")?
            .replace(" %u", "")
            .replace(" %U", "");

        Ok(SystemApp {
            name,
            exec,
            icon: AppIcon::from(icon.context("failed to get Icon")?),
        })
    }

    pub(crate) fn exec(&self) -> Result<()> {
        hyprctl::dispatch(format!("exec {}", self.exec))
            .with_context(|| format!("failed to exec {}", self.exec))
    }
}
fn desktop_files() -> Result<Vec<PathBuf>> {
    let mut out = vec![];

    for dir in [global_dir(), user_dir()?] {
        if let Ok(readdir) = std::fs::read_dir(&dir) {
            for entry in readdir {
                let file = entry.with_context(|| format!("failed to get file list of {dir}"))?;
                let path = file.path();
                if path.extension().is_some_and(|ext| ext == "desktop") {
                    out.push(path);
                }
            }
        }
    }

    Ok(out)
}

fn global_dir() -> String {
    String::from("/usr/share/applications")
}

fn user_dir() -> Result<String> {
    Ok(format!(
        "{}/.local/share/applications",
        std::env::var("HOME").context("no $HOME")?
    ))
}

impl From<String> for AppIcon {
    fn from(s: String) -> Self {
        if s.starts_with('/') {
            AppIcon::IconPath(s.into())
        } else {
            AppIcon::IconName(s.into())
        }
    }
}

use crate::{event::LauncherAppIcon, hyprctl};
use anyhow::{Context as _, Result};
use std::io::{BufRead as _, BufReader};

#[derive(Debug, Clone)]
pub(crate) struct DesktopFile {
    pub(crate) path: String,
    pub(crate) app_name: String,
    pub(crate) exec: String,
    pub(crate) icon: LauncherAppIcon,
}

// https://specifications.freedesktop.org/desktop-entry-spec/latest/exec-variables.html
const VARIABLES: &[&str] = &[
    "%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v", "%m",
];

impl DesktopFile {
    pub(crate) fn parse(path: &str) -> Result<DesktopFile> {
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
        let mut exec = exec.context("failed to get Exec")?;
        for var in VARIABLES {
            exec = exec.replace(var, "");
        }

        let icon = icon.context("failed to get Icon")?;
        let icon = if icon.starts_with('/') {
            LauncherAppIcon::IconPath(icon)
        } else {
            LauncherAppIcon::IconName(icon)
        };

        Ok(DesktopFile {
            path: path.to_string(),
            app_name: name,
            exec,
            icon,
        })
    }

    pub(crate) fn parse_many(iter: impl Iterator<Item = impl AsRef<str>>) -> Vec<Self> {
        let mut out = vec![];
        for filepath in iter {
            let filepath = filepath.as_ref();
            match DesktopFile::parse(filepath) {
                Ok(app) => out.push(app),
                Err(err) => log::warn!(
                    "Failed to parse desktop file {filepath}, skipping it: {:?}",
                    err
                ),
            }
        }
        out
    }

    pub(crate) fn exec(&self) -> Result<()> {
        hyprctl::dispatch(format!("exec {}", self.exec))
            .with_context(|| format!("failed to exec {}", self.exec))
    }
}

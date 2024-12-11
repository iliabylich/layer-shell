use crate::AppIcon;
use anyhow::{Context as _, Result};
use std::{collections::HashMap, path::PathBuf, process::Stdio};
use tokio::{
    io::{AsyncBufReadExt as _, BufReader},
    process::Command,
};

#[derive(Debug, Clone)]
pub(crate) struct SystemApp {
    pub(crate) name: String,
    pub(crate) exec: String,
    pub(crate) icon: AppIcon,
}

impl SystemApp {
    pub(crate) async fn parse_all() -> Result<Vec<SystemApp>> {
        let filepaths = desktop_files().await?;

        let mut apps = HashMap::new();

        for path in filepaths {
            match SystemApp::parse(&path).await {
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

    async fn parse(path: &PathBuf) -> Result<SystemApp> {
        let f = tokio::fs::File::open(path)
            .await
            .with_context(|| format!("failed to open {path:?}"))?;

        let mut lines = BufReader::new(f).lines();
        let mut in_desktop_entry_section = false;
        let mut name = None;
        let mut exec = None;
        let mut icon = None;
        while let Ok(Some(line)) = lines.next_line().await {
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

    pub(crate) fn exec(&self) {
        let parts = self.exec.split(" ").map(|s| s.trim()).collect::<Vec<_>>();
        let child = Command::new(parts[0])
            .args(&parts[1..])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        if let Err(err) = child {
            log::error!("Failed to spawn {}: {:?}", self.exec, err);
        }
    }
}

async fn desktop_files() -> Result<Vec<PathBuf>> {
    let mut out = vec![];

    for dir in [global_dir(), user_dir()?] {
        if let Ok(mut readdir) = tokio::fs::read_dir(&dir).await {
            while let Some(file) = readdir
                .next_entry()
                .await
                .with_context(|| format!("failed to get file list of {dir}"))?
            {
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

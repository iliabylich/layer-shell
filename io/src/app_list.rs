use crate::{App, AppIcon, Command, Event};
use anyhow::{Context, Result};
use layer_shell_utils::global;
use std::{collections::HashMap, path::PathBuf};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::mpsc::Sender,
};

struct AppList {
    selected_idx: usize,
    apps: Vec<DesktopApp>,
    pattern: String,
    tx: Sender<Event>,
}
global!(APP_LIST, AppList);

impl AppList {
    const MAX_ITEMS: usize = 5;

    async fn new(tx: Sender<Event>) -> Result<Self> {
        let apps = get_all_apps().await?;

        Ok(Self {
            selected_idx: 0,
            apps,
            pattern: String::new(),
            tx,
        })
    }

    async fn go_up(&mut self) {
        if self.selected_idx == 0 {
            return;
        }
        self.selected_idx = std::cmp::max(0, self.selected_idx - 1);
        self.emit().await;
    }
    async fn go_down(&mut self) {
        self.selected_idx = std::cmp::min(Self::MAX_ITEMS - 1, self.selected_idx + 1);
        self.emit().await;
    }
    async fn set_search(&mut self, pattern: &str) {
        self.selected_idx = 0;
        self.pattern = pattern.to_string();
        self.emit().await;
    }
    async fn exec_selected(&mut self) {
        if let Some(app) = self.visible_apps().await.get(self.selected_idx) {
            let parts = app.exec.split(" ").map(|s| s.trim()).collect::<Vec<_>>();
            let child = tokio::process::Command::new(parts[0])
                .args(&parts[1..])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();

            if let Err(err) = child {
                log::error!("Failed to spawn {}: {}", app.exec, err);
            }
        }
    }
    async fn reset(&mut self) {
        self.pattern = String::new();
        self.selected_idx = 0;
        match get_all_apps().await {
            Ok(apps) => self.apps = apps,
            Err(err) => log::error!("failed to refresh app list: {}\n{}", err, err.backtrace()),
        }
        self.emit().await;
    }

    async fn emit(&self) {
        let apps = self
            .visible_apps()
            .await
            .into_iter()
            .enumerate()
            .map(|(idx, app)| App {
                name: app.name,
                selected: idx == self.selected_idx,
                icon: app.icon,
            })
            .collect::<Vec<_>>();

        if self.tx.send(Event::AppList(apps)).await.is_err() {
            log::error!("failed to send AppList event");
        }
    }

    async fn visible_apps(&self) -> Vec<DesktopApp> {
        let apps = vec![DesktopApp {
            name: String::from("Google Chrome (wayland, gtk4)"),
            exec: String::from(
                "google-chrome-stable --gtk-version=4 --ozone-platform-hint=wayland",
            ),
            icon: AppIcon::IconName(String::from("google-chrome")),
        }]
        .into_iter();

        let desktop_apps = self
            .apps
            .iter()
            .filter(|app| !matches!(app.name.as_str(), "Google Chrome"))
            .cloned();

        let apps = apps.chain(desktop_apps);

        let pattern = self.pattern.to_lowercase();
        apps.filter(|app| app.name.to_lowercase().contains(&pattern))
            .take(Self::MAX_ITEMS)
            .collect()
    }
}

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("Memory model error: {}\n{}", err, err.backtrace());
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    let app_list = AppList::new(tx).await?;
    app_list.emit().await;
    APP_LIST::set(app_list);

    Ok(())
}

async fn get_all_apps() -> Result<Vec<DesktopApp>> {
    let dirs = vec![
        String::from("/usr/share/applications"),
        format!(
            "{}/.local/share/applications",
            std::env::var("HOME").context("no $HOME")?
        ),
    ];

    let filepaths = collect_all_filepaths(&dirs).await?;
    let apps = collect_all_apps(&filepaths).await?;

    Ok(apps)
}

async fn collect_all_filepaths(dirs: &[String]) -> Result<Vec<PathBuf>> {
    let mut out = vec![];

    for dir in dirs.iter() {
        if let Ok(mut readdir) = tokio::fs::read_dir(dir).await {
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

async fn collect_all_apps(filepaths: &[PathBuf]) -> Result<Vec<DesktopApp>> {
    let mut apps = HashMap::new();

    for path in filepaths.iter() {
        match parse_file(path).await {
            Ok(desktop_entry) => {
                apps.insert(desktop_entry.name.clone(), desktop_entry);
            }
            Err(err) => {
                log::warn!("Failed to parse {:?}: {}\n{}", path, err, err.backtrace());
            }
        }
    }

    let mut apps = apps.into_values().collect::<Vec<_>>();
    apps.sort_unstable_by(|app1, app2| app1.name.cmp(&app2.name));

    Ok(apps)
}

#[derive(Debug, Clone)]
struct DesktopApp {
    name: String,
    exec: String,
    icon: AppIcon,
}

impl From<String> for AppIcon {
    fn from(s: String) -> Self {
        if s.starts_with('/') {
            AppIcon::IconPath(s)
        } else {
            AppIcon::IconName(s)
        }
    }
}

async fn parse_file(path: &PathBuf) -> Result<DesktopApp> {
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

    Ok(DesktopApp {
        name: name.context("failed to get Name")?,
        exec: exec.context("failed to get Exec")?,
        icon: AppIcon::from(icon.context("failed to get Icon")?),
    })
}

pub(crate) async fn on_command(command: &Command) {
    let app_list = APP_LIST::get();

    match command {
        Command::LauncherReset => app_list.reset().await,
        Command::LauncherGoUp => app_list.go_up().await,
        Command::LauncherGoDown => app_list.go_down().await,
        Command::LauncherSetSearch(search) => app_list.set_search(search).await,
        Command::LauncherExecSelected => app_list.exec_selected().await,

        _ => {}
    }
}

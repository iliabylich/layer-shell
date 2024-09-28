use crate::models::Event;
use std::collections::HashSet;
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    net::UnixStream,
    sync::mpsc::Sender,
};

pub(crate) async fn spawn(tx: Sender<Event>) {
    let mut lines = match connect_to_hyprland().await {
        Some(lines) => lines,
        None => {
            eprintln!("Failed to connect to Hyprland");
            return;
        }
    };

    let mut workspaces = Workspaces::new().await;
    tx.send(Event::from(&workspaces)).await.unwrap();

    let lang = get_language().await;
    tx.send(Event::Language { lang }).await.unwrap();

    while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(hyprland_event) = HyprlandEvent::try_from(line) {
            match hyprland_event {
                HyprlandEvent::CreateWorkspace(_)
                | HyprlandEvent::DestroyWorkspace(_)
                | HyprlandEvent::Workspace(_) => {
                    workspaces.apply(hyprland_event);
                    tx.send(Event::from(&workspaces)).await.unwrap();
                }
                HyprlandEvent::LanguageChanged(lang) => {
                    tx.send(Event::Language { lang }).await.unwrap();
                }
            }
        }
    }
}

async fn connect_to_hyprland() -> Option<Lines<BufReader<UnixStream>>> {
    let socket_path = format!(
        "{}/hypr/{}/.socket2.sock",
        std::env::var("XDG_RUNTIME_DIR").expect("no XDG_RUNTIME_DIR variable"),
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("no HYPRLAND_INSTANCE_SIGNATURE, are you in Hyprland?"),
    );

    let socket = match UnixStream::connect(&socket_path).await {
        Ok(socket) => socket,
        Err(err) => {
            eprintln!("Failed to connect to Hyprland socket: {}", err);
            return None;
        }
    };
    let buffered = BufReader::new(socket);
    let lines = buffered.lines();
    Some(lines)
}

#[derive(Clone, Debug)]
pub(crate) enum HyprlandEvent {
    CreateWorkspace(usize),
    DestroyWorkspace(usize),
    Workspace(usize),
    LanguageChanged(String),
}

impl TryFrom<String> for HyprlandEvent {
    type Error = ();

    fn try_from(line: String) -> Result<Self, Self::Error> {
        let (event, payload) = line.split_once(">>").ok_or(())?;

        let payload_as_usize = || {
            payload
                .parse::<usize>()
                .map_err(|_| ())
                .inspect_err(|_| eprintln!("non integer payload of event {event}: {payload}"))
        };

        match event {
            "createworkspace" => Ok(Self::CreateWorkspace(payload_as_usize()?)),
            "destroyworkspace" => Ok(Self::DestroyWorkspace(payload_as_usize()?)),
            "workspace" => Ok(Self::Workspace(payload_as_usize()?)),
            "activelayout" => match payload.split(",").last() {
                Some(lang) => Ok(Self::LanguageChanged(lang.to_string())),
                None => {
                    eprintln!("unexpected payload of activelayout: {payload}");
                    Err(())
                }
            },
            _ => Err(()),
        }
    }
}

struct Workspaces {
    workspace_ids: HashSet<usize>,
    active_id: usize,
}
impl Workspaces {
    async fn new() -> Self {
        #[derive(serde::Deserialize)]
        pub(crate) struct Workspace {
            pub(crate) id: usize,
        }
        let stdout = exec_hyprctl("workspaces").await;
        let workspaces: Vec<Workspace> = serde_json::from_str(&stdout).unwrap();

        let stdout = exec_hyprctl("activeworkspace").await;
        let active_workspace: Workspace = serde_json::from_str(&stdout).unwrap();

        Self {
            workspace_ids: HashSet::from_iter(workspaces.into_iter().map(|w| w.id)),
            active_id: active_workspace.id,
        }
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

async fn get_language() -> String {
    #[derive(serde::Deserialize)]
    pub(crate) struct Devices {
        pub(crate) keyboards: Vec<Keyboard>,
    }
    #[derive(serde::Deserialize)]
    pub(crate) struct Keyboard {
        pub(crate) main: bool,
        pub(crate) active_keymap: String,
    }

    let stdout = exec_hyprctl("devices").await;
    let devices: Devices = serde_json::from_str(&stdout).unwrap();

    let main_keyboard = devices
        .keyboards
        .into_iter()
        .find(|keyboard| keyboard.main)
        .unwrap();

    main_keyboard.active_keymap
}

async fn exec_hyprctl(command: &str) -> String {
    let stdout = tokio::process::Command::new("hyprctl")
        .args([command, "-j"])
        .output()
        .await
        .unwrap()
        .stdout;
    String::from_utf8(stdout)
        .unwrap_or_else(|_| panic!("hyprctl {command} -j returned non-utf-8 stdout"))
}

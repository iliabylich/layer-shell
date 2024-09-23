use std::path::Path;

use gtk4::{
    gio::{Cancellable, DataInputStream, SocketClient, UnixSocketAddress},
    glib::Priority,
    prelude::{DataInputStreamExtManual, IOStreamExt, SocketClientExt},
};

use crate::utils::{exec_async, singleton, Singleton};

pub(crate) struct HyprlandClient {
    handlers: Vec<Box<dyn Fn(HyprlandEvent)>>,
}
singleton!(HyprlandClient);

fn socker_path() -> String {
    format!(
        "{}/hypr/{}/.socket2.sock",
        std::env::var("XDG_RUNTIME_DIR").unwrap(),
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap(),
    )
}

#[derive(Clone, Debug)]
pub(crate) enum HyprlandEvent {
    CreateWorkspace(usize),
    DestroyWorkspace(usize),
    Workspace(usize),
    LanguageChanged(String),
}

impl HyprlandClient {
    pub(crate) fn spawn() {
        Self::set(Self { handlers: vec![] });

        gtk4::glib::spawn_future_local(async {
            let unix_socket = UnixSocketAddress::new(Path::new(&socker_path()));
            let socket = SocketClient::new();
            let connection = socket.connect(&unix_socket, Cancellable::NONE).unwrap();
            let stream = DataInputStream::builder()
                .base_stream(&connection.input_stream())
                .close_base_stream(true)
                .build();

            loop {
                let line = stream.read_line_future(Priority::DEFAULT).await.unwrap();
                let line = std::str::from_utf8(line.as_ref()).unwrap();

                let (event, payload) = line.split_once(">>").unwrap();

                let event = match event {
                    "createworkspace" => HyprlandEvent::CreateWorkspace(payload.parse().unwrap()),
                    "destroyworkspace" => HyprlandEvent::DestroyWorkspace(payload.parse().unwrap()),
                    "workspace" => HyprlandEvent::Workspace(payload.parse().unwrap()),
                    "activelayout" => {
                        let lang = payload.split(",").last().unwrap();
                        HyprlandEvent::LanguageChanged(lang.to_string())
                    }
                    _ => continue,
                };

                for handler in Self::get().handlers.iter() {
                    handler(event.clone());
                }
            }
        });
    }

    pub(crate) fn subscribe<F>(f: F)
    where
        F: Fn(HyprlandEvent) + 'static,
    {
        Self::get().handlers.push(Box::new(f))
    }

    pub(crate) async fn get_workspaces() -> Vec<Workspace> {
        let json = exec_async(&["hyprctl", "workspaces", "-j"]).await;
        serde_json::from_str(&json).unwrap()
    }

    pub(crate) async fn get_active_workspace() -> Workspace {
        let json = exec_async(&["hyprctl", "activeworkspace", "-j"]).await;
        serde_json::from_str(&json).unwrap()
    }

    pub(crate) async fn get_devices() -> Devices {
        let json = exec_async(&["hyprctl", "devices", "-j"]).await;
        serde_json::from_str(&json).unwrap()
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct Workspace {
    pub(crate) id: usize,
}

#[derive(serde::Deserialize)]
pub(crate) struct Devices {
    pub(crate) keyboards: Vec<Keyboard>,
}

#[derive(serde::Deserialize)]
pub(crate) struct Keyboard {
    pub(crate) main: bool,
    pub(crate) active_keymap: String,
}

use crate::utils::{exec_async, singleton};
use anyhow::{Context, Result};
use gtk4::{
    gio::{Cancellable, DataInputStream, SocketClient, UnixSocketAddress},
    glib::Priority,
    prelude::{DataInputStreamExtManual, IOStreamExt, SocketClientExt},
};
use std::path::Path;

pub(crate) struct HyprlandClient {
    handlers: Vec<Box<dyn Fn(HyprlandEvent)>>,
}
singleton!(HyprlandClient);

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
            match connect_to_hyprland_socket() {
                Ok(stream) => loop {
                    match read_line_from_stream(&stream).await {
                        Ok(line) => {
                            if let Ok(event) = HyprlandEvent::try_from(line.as_str()) {
                                Self::dispatch(event);
                            }
                        }
                        Err(err) => {
                            eprintln!("failed to read line from Hyprland socket:\n{}", err);
                        }
                    }
                },
                Err(_) => todo!(),
            }
        });
    }

    pub(crate) fn subscribe<F>(f: F)
    where
        F: Fn(HyprlandEvent) + 'static,
    {
        this().handlers.push(Box::new(f))
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

    fn dispatch(event: HyprlandEvent) {
        for handler in this().handlers.iter() {
            handler(event.clone());
        }
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

fn connect_to_hyprland_socket() -> Result<DataInputStream> {
    let socket_path = format!(
        "{}/hypr/{}/.socket2.sock",
        std::env::var("XDG_RUNTIME_DIR").expect("no XDG_RUNTIME_DIR variable"),
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("no HYPRLAND_INSTANCE_SIGNATURE, are you in Hyprland?"),
    );

    fn g<T>(value: T) -> &'static T {
        Box::leak(Box::new(value))
    }

    let unix_socket = g(UnixSocketAddress::new(Path::new(&socket_path)));
    let socket = g(SocketClient::new());
    let connection = g(socket
        .connect(unix_socket, Cancellable::NONE)
        .context("failed to connect to Hyprland socket")?);
    let stream = DataInputStream::builder()
        .base_stream(&connection.input_stream())
        .close_base_stream(true)
        .build();

    Ok(stream)
}

async fn read_line_from_stream(stream: &DataInputStream) -> Result<String> {
    let line = stream
        .read_line_future(Priority::DEFAULT)
        .await
        .context("failed to read line")?;
    let line = std::str::from_utf8(line.as_ref()).context("non-utf-8 line from Hyprland socket")?;
    Ok(line.to_string())
}

impl TryFrom<&str> for HyprlandEvent {
    type Error = ();

    fn try_from(line: &str) -> Result<Self, Self::Error> {
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

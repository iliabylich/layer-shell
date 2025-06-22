use std::collections::HashSet;

use crate::env::{hyprland_instance_signature, xdg_runtime_dir};
use anyhow::{Context as _, Result, bail};
use serde::Deserialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

pub(crate) struct Writer;

impl Writer {
    async fn call(cmd: impl AsRef<str>) -> Result<String> {
        let path = format!(
            "{}/hypr/{}/.socket.sock",
            xdg_runtime_dir()?,
            hyprland_instance_signature()?
        );
        let mut socket = UnixStream::connect(&path)
            .await
            .context("failed to open writer socket")?;

        let cmd = cmd.as_ref();
        socket.write_all(cmd.as_bytes()).await?;

        let mut out = vec![];
        socket
            .read_to_end(&mut out)
            .await
            .context("failed to read to end")?;
        let out = String::from_utf8(out).context("non-utf-8 response from hyprland socket")?;

        Ok(out)
    }

    pub(crate) async fn dispatch(cmd: impl AsRef<str>) -> Result<()> {
        let res = Self::call(format!("dispatch {}", cmd.as_ref())).await?;

        if res != "ok" {
            bail!(
                "invalid response from hyprctl dispatch: expected 'ok', got {:?}",
                res
            );
        }
        Ok(())
    }

    pub(crate) async fn get_workspaces_list() -> Result<HashSet<usize>> {
        let json = Self::call("[[BATCH]]j/workspaces").await?;
        let workspaces: Vec<Workspace> =
            serde_json::from_str(&json).context("malformed workspaces response")?;
        Ok(workspaces.into_iter().map(|w| w.id).collect())
    }

    pub(crate) async fn get_active_workspace() -> Result<usize> {
        let json = Self::call("[[BATCH]]j/activeworkspace").await?;
        let workspace: Workspace =
            serde_json::from_str(&json).context("malformed activeworkspace response")?;
        Ok(workspace.id)
    }

    pub(crate) async fn get_language() -> Result<String> {
        let json = Self::call("[[BATCH]]j/devices").await?;
        let devices: Devices = serde_json::from_str(&json).context("malformed devices response")?;
        let main_keyboard = devices
            .keyboards
            .into_iter()
            .find(|keyboard| keyboard.main)
            .context("expected at least one hyprland device")?;

        Ok(main_keyboard.active_keymap)
    }
}

#[derive(Deserialize)]
pub(crate) struct Workspace {
    id: usize,
}
#[derive(Deserialize)]
struct Devices {
    keyboards: Vec<Keyboard>,
}
#[derive(Deserialize)]
struct Keyboard {
    main: bool,
    active_keymap: String,
}

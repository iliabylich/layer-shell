use crate::modules::hyprland::resources::{WriterReply, WriterResource};
use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::borrow::Cow;

pub(crate) struct DevicesResource;
impl WriterResource for DevicesResource {
    fn command(&self) -> Cow<'static, str> {
        Cow::Borrowed("[[BATCH]]j/devices")
    }

    fn parse(&self, json: &str) -> Result<WriterReply> {
        #[derive(Deserialize)]
        struct Devices {
            keyboards: Vec<Keyboard>,
        }
        #[derive(Deserialize)]
        struct Keyboard {
            main: bool,
            active_keymap: String,
        }

        let devices: Devices = serde_json::from_str(json).context("malformed devices response")?;

        let active_keymap = devices
            .keyboards
            .into_iter()
            .find(|keyboard| keyboard.main)
            .context("expected at least one hyprland device")?
            .active_keymap;

        Ok(WriterReply::ActiveKeymap(active_keymap))
    }
}

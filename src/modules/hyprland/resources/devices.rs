use crate::{
    modules::hyprland::{resources::WriterResource, state::HyprlandDiff},
    utils::StringRef,
};
use anyhow::{Context as _, Result};
use serde::Deserialize;

pub(crate) struct DevicesResource;
impl WriterResource for DevicesResource {
    fn command(&self) -> StringRef {
        StringRef::new("[[BATCH]]j/devices")
    }

    fn parse(&self, json: &str) -> Result<Option<HyprlandDiff>> {
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

        Ok(Some(HyprlandDiff::SetLanguage(StringRef::new(
            active_keymap.as_str(),
        ))))
    }
}

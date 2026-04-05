use crate::{
    modules::hyprland::{resources::WriterResource, state::HyprlandDiff},
    utils::StringRef,
};
use anyhow::{Context as _, Result};
use serde::Deserialize;

pub(crate) struct CapsLockResource;
impl WriterResource for CapsLockResource {
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
            #[serde(rename = "capsLock")]
            caps_lock: bool,
        }

        let devices: Devices = serde_json::from_str(json).context("malformed devices response")?;
        let main_keyboard = devices
            .keyboards
            .into_iter()
            .find(|keyboard| keyboard.main)
            .context("expected at least one hyprland device")?;

        Ok(Some(HyprlandDiff::SetCapsLockEnabled(
            main_keyboard.caps_lock,
        )))
    }
}

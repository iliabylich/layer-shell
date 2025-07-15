use crate::writer::Writer;
use anyhow::{Context as _, Result};
use serde::Deserialize;

pub struct CapsLock;

impl CapsLock {
    pub async fn is_enabled() -> Result<bool> {
        let json = Writer::call("[[BATCH]]j/devices").await?;

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

        let devices: Devices = serde_json::from_str(&json).context("malformed devices response")?;
        let main_keyboard = devices
            .keyboards
            .into_iter()
            .find(|keyboard| keyboard.main)
            .context("expected at least one hyprland device")?;

        Ok(main_keyboard.caps_lock)
    }
}

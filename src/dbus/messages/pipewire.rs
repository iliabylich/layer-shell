use super::{PropertiesChanged, value_is};
use crate::dbus::types::{Message, Value};
use anyhow::{Context as _, Result, ensure};

#[derive(Debug)]
pub(crate) struct VolumeChanged {
    volume: u32,
}

impl TryFrom<&Message> for VolumeChanged {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
        let PropertiesChanged {
            path,
            interface,
            mut changes,
        } = PropertiesChanged::try_from(message)?;

        ensure!(path == "/org/local/PipewireDBus");
        ensure!(interface == "org.local.PipewireDBus");

        let volume = changes.remove("Volume").context("unrelaterd")?;
        value_is!(volume, Value::UInt32(volume));

        Ok(Self { volume })
    }
}

#[derive(Debug)]
pub(crate) struct MuteChanged {
    muted: bool,
}

impl TryFrom<&Message> for MuteChanged {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
        let PropertiesChanged {
            path,
            interface,
            mut changes,
        } = PropertiesChanged::try_from(message)?;

        ensure!(path == "/org/local/PipewireDBus");
        ensure!(interface == "org.local.PipewireDBus");

        let muted = changes.remove("Muted").context("unrelaterd")?;
        value_is!(muted, Value::Bool(muted));

        Ok(Self { muted })
    }
}

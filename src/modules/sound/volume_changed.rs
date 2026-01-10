use crate::dbus::{
    messages::{interface_is, org_freedesktop_dbus::PropertiesChanged, path_is, value_is},
    types::Value,
};
use anyhow::{Context as _, Result};

#[derive(Debug)]
pub(crate) struct VolumeChanged {
    pub(crate) volume: u32,
}

impl TryFrom<&PropertiesChanged<'_>> for VolumeChanged {
    type Error = anyhow::Error;

    fn try_from(
        PropertiesChanged {
            path,
            interface,
            changes,
        }: &PropertiesChanged,
    ) -> Result<Self> {
        path_is!(path, "/org/local/PipewireDBus");
        interface_is!(interface, "org.local.PipewireDBus");

        let volume = changes.get("Volume").context("unrelaterd")?;
        value_is!(volume, Value::UInt32(volume));

        Ok(Self { volume: *volume })
    }
}

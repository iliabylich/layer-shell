use crate::dbus::{
    messages::{interface_is, org_freedesktop_dbus::PropertiesChanged, path_is, value_is},
    types::Value,
};
use anyhow::{Context as _, Result};

#[derive(Debug)]
pub(crate) struct MuteChanged {
    pub(crate) muted: bool,
}

impl TryFrom<&PropertiesChanged<'_>> for MuteChanged {
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

        let muted = changes.get("Muted").context("unrelaterd")?;
        value_is!(muted, Value::Bool(muted));

        Ok(Self { muted: *muted })
    }
}

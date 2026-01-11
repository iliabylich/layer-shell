use crate::dbus::{
    Message,
    messages::{destination_is, introspect::IntrospectRequest, path_is},
};
use anyhow::Result;

pub(crate) struct ControlIntrospectRequest {
    pub(crate) sender: String,
    pub(crate) serial: u32,
}

impl TryFrom<&Message> for ControlIntrospectRequest {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
        let IntrospectRequest {
            destination,
            path,
            sender,
            serial,
        } = IntrospectRequest::try_from(message)?;

        destination_is!(destination, "org.me.LayerShellTmpControl");
        path_is!(path, "/");
        Ok(Self {
            sender: sender.to_string(),
            serial,
        })
    }
}

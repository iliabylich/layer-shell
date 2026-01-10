use crate::dbus::{
    Message,
    messages::{body_is, destination_is, interface_is, message_is, path_is},
};
use anyhow::Result;

#[derive(Debug)]
pub(crate) enum ControlRequest {
    CapsLockToggled,
    Exit,
    ReloadStyles,
    ToggleSessionScreen,
}

pub(crate) enum AnyControlRequest {
    Known(ControlRequest),
    Unknown,
}

impl TryFrom<&Message> for AnyControlRequest {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
        message_is!(
            message,
            Message::MethodCall {
                path,
                member,
                interface,
                destination,
                body,
                ..
            }
        );

        path_is!(path, "/");
        destination_is!(destination.as_deref(), Some("org.me.LayerShellTmpControl"));
        interface_is!(interface.as_deref(), Some("org.me.LayerShellControl"));
        body_is!(body, []);

        let this = match member.as_ref() {
            "CapsLockToggled" => Self::Known(ControlRequest::CapsLockToggled),
            "Exit" => Self::Known(ControlRequest::Exit),
            "ReloadStyles" => Self::Known(ControlRequest::ReloadStyles),
            "ToggleSessionScreen" => Self::Known(ControlRequest::ToggleSessionScreen),
            _ => Self::Unknown,
        };

        Ok(this)
    }
}

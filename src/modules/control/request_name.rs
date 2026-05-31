use dbus::{EncodeError, messages::org_freedesktop_dbus::RequestName, messaging::DBusEncode};

pub(crate) struct RequestNameLayerShellControl;

impl DBusEncode for RequestNameLayerShellControl {
    type Args<'a> = ();

    fn encode<'a>((): Self::Args<'_>, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        RequestName::encode(buf, "org.me.LayerShellControl")
    }
}

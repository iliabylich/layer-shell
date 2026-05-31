use dbus::{Conf, DBusError, IncomingValue, messaging::property::Property, value_is};

#[derive(Clone)]
pub(crate) struct Volume;
impl Property for Volume {
    type Output<'a> = u32;

    const DESTINATION: Conf<str, Self> = Conf::constant("org.local.PipewireDBus");
    const PATH: Conf<str, Self> = Conf::constant("/org/local/PipewireDBus");
    const INTERFACE: Conf<str, Self> = Conf::constant("org.local.PipewireDBus");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("Volume");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError> {
        value_is!(value, IncomingValue::UInt32(mut value));
        if value == 99 {
            value = 100;
        }
        Ok(value)
    }
}

#[derive(Clone)]
pub(crate) struct Muted;
impl Property for Muted {
    type Output<'a> = bool;

    const DESTINATION: Conf<str, Self> = Conf::constant("org.local.PipewireDBus");
    const PATH: Conf<str, Self> = Conf::constant("/org/local/PipewireDBus");
    const INTERFACE: Conf<str, Self> = Conf::constant("org.local.PipewireDBus");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("Muted");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError> {
        value_is!(value, IncomingValue::Bool(value));
        Ok(value)
    }
}

use dbus::arg;

#[derive(Debug)]
pub(crate) struct DBusNameOwnerChanged {
    pub(crate) name: String,
    pub(crate) old_owner: String,
    pub(crate) new_owner: String,
}

impl arg::ReadAll for DBusNameOwnerChanged {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(DBusNameOwnerChanged {
            name: i.read()?,
            old_owner: i.read()?,
            new_owner: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for DBusNameOwnerChanged {
    const NAME: &'static str = "NameOwnerChanged";
    const INTERFACE: &'static str = "org.freedesktop.DBus";
}

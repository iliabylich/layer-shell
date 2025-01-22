use dbus::arg;

#[derive(Debug)]
pub(crate) struct DBusNameOwnerChanged {
    pub(crate) name: String,
    pub(crate) old_owner: String,
    pub(crate) new_owner: String,
}

impl arg::AppendAll for DBusNameOwnerChanged {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.name, i);
        arg::RefArg::append(&self.old_owner, i);
        arg::RefArg::append(&self.new_owner, i);
    }
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

impl DBusNameOwnerChanged {
    pub(crate) fn is_remove(&self) -> bool {
        self.name == self.old_owner && self.new_owner.is_empty()
    }
}

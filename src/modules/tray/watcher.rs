use crate::dbus::OrgKdeStatusNotifierWatcher;

pub(crate) struct Watcher {
    pub(crate) new_item: Option<String>,
}

impl Watcher {
    pub(crate) fn new() -> Self {
        Self { new_item: None }
    }

    pub(crate) fn pop_new_item(&mut self) -> Option<String> {
        self.new_item.take()
    }
}

impl OrgKdeStatusNotifierWatcher for Watcher {
    fn register_status_notifier_item(&mut self, path: String) -> Result<(), dbus::MethodErr> {
        self.new_item = Some(path);

        Ok(())
    }

    fn register_status_notifier_host(&mut self, _service: String) -> Result<(), dbus::MethodErr> {
        Ok(())
    }

    fn registered_status_notifier_items(&self) -> Result<Vec<String>, dbus::MethodErr> {
        Ok(vec![])
    }

    fn is_status_notifier_host_registered(&self) -> Result<bool, dbus::MethodErr> {
        Ok(true)
    }

    fn protocol_version(&self) -> Result<i32, dbus::MethodErr> {
        Ok(42)
    }
}

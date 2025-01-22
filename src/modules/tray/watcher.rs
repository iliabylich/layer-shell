use crate::dbus::OrgKdeStatusNotifierWatcher;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub(crate) struct Watcher {
    sender: Sender<WatcherData>,
}

#[derive(Debug)]
pub(crate) enum WatcherData {
    StatusNotifierItem { service: String, path: String },
    CanonicalDBusMenu { service: String },
}

impl Watcher {
    pub(crate) fn new(sender: Sender<WatcherData>) -> Self {
        Self { sender }
    }
}

impl OrgKdeStatusNotifierWatcher for Watcher {
    fn register_status_notifier_item(
        &mut self,
        path: String,
        ctx: &dbus_crossroads::Context,
    ) -> Result<(), dbus::MethodErr> {
        if let Some(service) = ctx.message().sender() {
            let service = service.to_string();

            let data = if service == path {
                WatcherData::CanonicalDBusMenu { service }
            } else {
                WatcherData::StatusNotifierItem { service, path }
            };

            if let Err(err) = self.sender.send(data) {
                log::error!("channel closed: {:?}", err);
            }
        }

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

use crate::{
    dbus::OrgKdeStatusNotifierWatcher,
    modules::tray::channel::{Command, CHANNEL},
};

#[derive(Debug)]
pub(crate) struct Watcher;

impl OrgKdeStatusNotifierWatcher for Watcher {
    fn register_status_notifier_item(
        &mut self,
        path: String,
        ctx: &dbus_crossroads::Context,
    ) -> Result<(), dbus::MethodErr> {
        if let Some(service) = ctx.message().sender().map(|s| s.to_string()) {
            CHANNEL.emit(Command::ServiceAdded { service, path });
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

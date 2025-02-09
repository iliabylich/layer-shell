use crate::{dbus::OrgKdeStatusNotifierWatcher, modules::tray::channel::TrayCommand};
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub(crate) struct Watcher {
    pub(crate) tx: Sender<TrayCommand>,
}

impl OrgKdeStatusNotifierWatcher for Watcher {
    fn register_status_notifier_item(
        &mut self,
        path: String,
        ctx: &dbus_crossroads::Context,
    ) -> Result<(), dbus::MethodErr> {
        if let Some(service) = ctx.message().sender().map(|s| s.to_string()) {
            if let Err(err) = self.tx.send(TrayCommand::Added { service, path }) {
                log::error!("failed to send TrayCommand::Added event: {:?}", err);
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

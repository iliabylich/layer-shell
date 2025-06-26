use std::sync::Arc;

use crate::dbus_event::DBusEvent;
use anyhow::Result;
use futures::Stream;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use zbus::{Connection, interface, object_server::SignalEmitter};

pub(crate) struct StatusNotifierWatcher {
    tx: UnboundedSender<DBusEvent>,
}

impl StatusNotifierWatcher {
    fn new() -> (Self, UnboundedReceiverStream<DBusEvent>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let stream = UnboundedReceiverStream::new(rx);

        (Self { tx }, stream)
    }

    pub(crate) async fn into_stream(conn: Connection) -> Result<impl Stream<Item = DBusEvent>> {
        let (iface, stream) = StatusNotifierWatcher::new();
        conn.object_server()
            .at("/StatusNotifierWatcher", iface)
            .await?;
        conn.request_name("org.kde.StatusNotifierWatcher").await?;
        Ok(stream)
    }
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    fn register_status_notifier_host(&self, _service: &str) {}

    fn register_status_notifier_item(&self, service: &str) {
        let event = DBusEvent::ServiceAdded(Arc::from(service.to_string()));
        if let Err(err) = self.tx.send(event) {
            log::error!(target: "Tray", "failed to process incoming service: {err:?}")
        }
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> zbus::fdo::Result<bool> {
        Ok(true)
    }

    #[zbus(property)]
    fn protocol_version(&self) -> zbus::fdo::Result<i32> {
        Ok(42)
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> zbus::fdo::Result<Vec<String>> {
        Ok(vec![])
    }

    #[zbus(signal)]
    async fn status_notifier_host_registered(
        &self,
        _emitter: SignalEmitter<'_>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_host_unregistered(
        &self,
        _emitter: SignalEmitter<'_>,
    ) -> zbus::Result<()>;
    #[zbus(signal)]

    async fn status_notifier_item_registered(
        &self,
        _emitter: SignalEmitter<'_>,
        service: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_item_unregistered(
        &self,
        _emitter: SignalEmitter<'_>,
        service: &str,
    ) -> zbus::Result<()>;
}

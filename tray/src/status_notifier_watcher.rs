use crate::{dbus_event::DBusEvent, stream_id::StreamId, tray_stream::TrayStream};
use anyhow::Result;
use futures::{StreamExt, stream::BoxStream};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use zbus::{Connection, interface, object_server::SignalEmitter};

pub(crate) struct StatusNotifierWatcher {
    tx: UnboundedSender<DBusEvent>,
}

#[async_trait::async_trait]
impl TrayStream for StatusNotifierWatcher {
    type Input = ();

    async fn stream(
        conn: &Connection,
        _: Self::Input,
    ) -> Result<(StreamId, BoxStream<'static, DBusEvent>)> {
        let id = StreamId::ServiceAdded;

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        conn.object_server()
            .at("/StatusNotifierWatcher", StatusNotifierWatcher { tx })
            .await?;
        conn.request_name("org.kde.StatusNotifierWatcher").await?;

        let stream = UnboundedReceiverStream::new(rx);

        Ok((id, stream.boxed()))
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

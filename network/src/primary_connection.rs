use crate::{
    multiplexer::StreamId, nm_event::NetworkManagerEvent, nm_stream::NmStream,
    primary_devices::PrimaryDevices,
};
use anyhow::Result;
use futures::{StreamExt, stream::BoxStream};
use zbus::proxy;
use zbus::{Connection, zvariant::OwnedObjectPath};

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    #[zbus(property)]
    fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    assume_defaults = true
)]
trait ActiveConnection {
    #[zbus(property)]
    fn type_(&self) -> zbus::Result<String>;
}

pub(crate) struct PrimaryConnection;

#[async_trait::async_trait]
impl NmStream for PrimaryConnection {
    const ID: &StreamId = &StreamId {
        name: "PRIMARY_CONNECTION",
        children: &[PrimaryDevices::ID],
    };

    type Input = ();

    async fn stream(
        conn: &Connection,
        _: Self::Input,
    ) -> Result<BoxStream<'static, NetworkManagerEvent>> {
        let proxy = NetworkManagerProxy::new(conn).await?;

        let path = proxy.primary_connection().await?;
        let event = NetworkManagerEvent::PrimaryConnection(path);
        let pre = futures::stream::once(async move { event });

        let post = proxy
            .receive_primary_connection_changed()
            .await
            .filter_map(|e| async move {
                let path = e.get().await.ok()?;
                Some(NetworkManagerEvent::PrimaryConnection(path))
            });

        Ok(pre.chain(post).boxed())
    }
}

impl PrimaryConnection {
    pub(crate) async fn is_wireless(path: OwnedObjectPath, conn: &Connection) -> Result<bool> {
        let proxy = ActiveConnectionProxy::builder(conn)
            .path(path)?
            .build()
            .await?;

        Ok(proxy.type_().await?.contains("wireless"))
    }
}

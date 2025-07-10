use crate::nm_event::NetworkManagerEvent;
use anyhow::Result;
use futures::{Stream, StreamExt};
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

impl PrimaryConnection {
    pub(crate) async fn stream(
        conn: &Connection,
    ) -> Result<impl Stream<Item = NetworkManagerEvent> + 'static> {
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

        Ok(pre.chain(post))
    }

    pub(crate) async fn is_wireless(path: OwnedObjectPath, conn: &Connection) -> Result<bool> {
        let proxy = ActiveConnectionProxy::builder(conn)
            .path(path)?
            .build()
            .await?;

        Ok(proxy.type_().await?.contains("wireless"))
    }
}

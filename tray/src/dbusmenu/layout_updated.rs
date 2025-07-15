use crate::{
    dbus_event::DBusEvent,
    stream_id::{ServiceStreamId, StreamId},
    tray_stream::TrayStream,
};
use anyhow::Result;
use futures::{StreamExt, stream::BoxStream};
use std::sync::Arc;
use zbus::{Connection, zvariant::OwnedObjectPath};

mod dbus {
    use zbus::proxy;

    #[proxy(interface = "com.canonical.dbusmenu", assume_defaults = true)]
    pub(crate) trait DBusMenu {
        #[zbus(signal)]
        fn layout_updated(&self, revision: u32, parent: i32) -> zbus::Result<()>;
    }
}

pub(crate) struct LayoutUpdated;

#[async_trait::async_trait]
impl TrayStream for LayoutUpdated {
    type Input = (Arc<str>, Arc<OwnedObjectPath>);

    async fn stream(
        conn: &Connection,
        (service, menu): Self::Input,
    ) -> Result<(StreamId, BoxStream<'static, DBusEvent>)> {
        let id = StreamId::ServiceStream {
            service: Arc::clone(&service),
            id: ServiceStreamId::LayoutUpdated,
        };

        let proxy = dbus::DBusMenuProxy::builder(conn)
            .destination(service.as_ref())?
            .path(menu.as_ref())?
            .build()
            .await?;

        let event = DBusEvent::LayoutUpdated {
            service: Arc::clone(&service),
            menu: Arc::clone(&menu),
        };
        let pre = futures::stream::once(async move { event });

        let post = proxy.receive_layout_updated().await?.filter_map(move |_| {
            let service = Arc::clone(&service);
            let menu = Arc::clone(&menu);
            async move { Some(DBusEvent::LayoutUpdated { service, menu }) }
        });

        Ok((id, pre.chain(post).boxed()))
    }
}

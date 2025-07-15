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
        fn items_properties_updated(
            &self,
            updated_props: Vec<(
                i32,
                std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
            )>,
            removed_props: Vec<(i32, Vec<&str>)>,
        ) -> zbus::Result<()>;
    }
}

pub(crate) struct ItemsPropertiesUpdated;

#[async_trait::async_trait]
impl TrayStream for ItemsPropertiesUpdated {
    type Input = (Arc<str>, Arc<OwnedObjectPath>);

    async fn stream(
        conn: &Connection,
        (service, menu): Self::Input,
    ) -> Result<(StreamId, BoxStream<'static, DBusEvent>)> {
        let proxy = dbus::DBusMenuProxy::builder(&conn)
            .destination(service.as_ref())?
            .path(menu.as_ref())?
            .build()
            .await?;

        let id = StreamId::ServiceStream {
            service: Arc::clone(&service),
            id: ServiceStreamId::ItemsPropertiesUpdated,
        };

        let stream = proxy
            .receive_items_properties_updated()
            .await?
            .filter_map(move |_| {
                let service = Arc::clone(&service);
                let menu = Arc::clone(&menu);
                async move { Some(DBusEvent::ItemsPropertiesUpdated { service, menu }) }
            })
            .boxed();

        Ok((id, stream))
    }
}

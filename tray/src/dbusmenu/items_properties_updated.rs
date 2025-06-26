use crate::{dbus_event::DBusEvent, dbusmenu::proxy::DBusMenuProxy, stream_id::StreamId};
use anyhow::Result;
use futures::{Stream, StreamExt};
use std::sync::Arc;
use zbus::{Connection, zvariant::OwnedObjectPath};

pub(crate) struct ItemsPropertiesUpdated;

impl ItemsPropertiesUpdated {
    pub(crate) async fn split(
        conn: Connection,
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    ) -> Result<(Option<DBusEvent>, StreamId, impl Stream<Item = DBusEvent>)> {
        let proxy = DBusMenuProxy::builder(&conn)
            .destination(service.to_string())?
            .path(menu.as_ref().to_string())?
            .build()
            .await?;

        let event = None;

        let stream_id = StreamId::ItemsPropertiesUpdated {
            service: Arc::clone(&service),
        };

        let stream = proxy
            .receive_items_properties_updated()
            .await?
            .filter_map(move |_| {
                let service = Arc::clone(&service);
                let menu = Arc::clone(&menu);
                async move {
                    Some(DBusEvent::ItemsPropertiesUpdated {
                        service: Arc::clone(&service),
                        menu: Arc::clone(&menu),
                    })
                }
            });

        Ok((event, stream_id, stream))
    }
}

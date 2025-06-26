use crate::{dbus_event::DBusEvent, dbusmenu::proxy::DBusMenuProxy, stream_id::StreamId};
use anyhow::Result;
use futures::{Stream, StreamExt};
use std::sync::Arc;
use zbus::{Connection, zvariant::OwnedObjectPath};

pub(crate) struct LayoutUpdated;

impl LayoutUpdated {
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

        let event = Some(DBusEvent::LayoutUpdated {
            service: Arc::clone(&service),
            menu: Arc::clone(&menu),
        });

        let stream_id = StreamId::LayoutUpdated {
            service: Arc::clone(&service),
        };

        let stream = proxy.receive_layout_updated().await?.filter_map(move |_| {
            let service = Arc::clone(&service);
            let menu = Arc::clone(&menu);
            async move {
                Some(DBusEvent::LayoutUpdated {
                    service: Arc::clone(&service),
                    menu: Arc::clone(&menu),
                })
            }
        });

        Ok((event, stream_id, stream))
    }
}

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
            parent_id: 0,
        });

        let stream_id = StreamId::LayoutUpdated {
            service: Arc::clone(&service),
        };

        let stream = proxy.receive_layout_updated().await?.filter_map(move |e| {
            let service = Arc::clone(&service);
            let menu = Arc::clone(&menu);
            async move {
                let args = e.args().ok()?;
                let parent_id = args.parent;
                Some(DBusEvent::LayoutUpdated {
                    service: Arc::clone(&service),
                    menu: Arc::clone(&menu),
                    parent_id,
                })
            }
        });

        Ok((event, stream_id, stream))
    }
}

use crate::{
    dbus_event::DBusEvent, status_notifier_item::proxy::StatusNotifierItemProxy,
    stream_id::StreamId,
};
use anyhow::Result;
use futures::{Stream, StreamExt};
use std::sync::Arc;
use zbus::Connection;

pub(crate) struct NewIcon;

impl NewIcon {
    pub(crate) async fn into_stream(
        conn: Connection,
        service: Arc<str>,
    ) -> Result<(StreamId, impl Stream<Item = DBusEvent>)> {
        let proxy = StatusNotifierItemProxy::builder(&conn)
            .destination(service.to_string())?
            .build()
            .await?;

        let stream_id = StreamId::NewIconNotified {
            service: Arc::clone(&service),
        };

        let stream = proxy.receive_new_icon().await?.filter_map(move |_e| {
            let service = Arc::clone(&service);
            async move {
                Some(DBusEvent::NewIconNotified {
                    service: Arc::clone(&service),
                })
            }
        });

        Ok((stream_id, stream))
    }
}

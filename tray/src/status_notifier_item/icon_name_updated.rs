use crate::{
    dbus_event::DBusEvent, status_notifier_item::proxy::StatusNotifierItemProxy,
    stream_id::StreamId,
};
use anyhow::{Context as _, Result};
use futures::{Stream, StreamExt};
use std::sync::Arc;
use zbus::Connection;

pub(crate) struct IconNameUpdated;

impl IconNameUpdated {
    pub(crate) async fn split(
        conn: Connection,
        service: Arc<str>,
    ) -> Result<(Result<DBusEvent>, StreamId, impl Stream<Item = DBusEvent>)> {
        let proxy = StatusNotifierItemProxy::builder(&conn)
            .destination(service.to_string())?
            .build()
            .await?;

        let event = proxy
            .icon_name()
            .await
            .context("failed to get IconName")
            .map(|icon_name| DBusEvent::IconNameChanged {
                service: Arc::clone(&service),
                icon_name,
            });

        let stream_id = StreamId::IconNameUpdated {
            service: Arc::clone(&service),
        };

        let stream = proxy
            .receive_icon_name_changed()
            .await
            .filter_map(move |e| {
                let service = Arc::clone(&service);
                async move {
                    let icon_name = e.get().await.ok()?;
                    Some(DBusEvent::IconNameChanged {
                        service: Arc::clone(&service),
                        icon_name,
                    })
                }
            });

        Ok((event, stream_id, stream))
    }
}

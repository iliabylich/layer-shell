use crate::{
    dbus_event::DBusEvent, status_notifier_item::proxy::StatusNotifierItemProxy,
    stream_id::StreamId,
};
use anyhow::{Context as _, Result};
use futures::{Stream, StreamExt};
use std::sync::Arc;
use zbus::Connection;

pub(crate) struct IconPixmapUpdate;

impl IconPixmapUpdate {
    pub(crate) async fn split(
        conn: Connection,
        service: Arc<str>,
    ) -> Result<(Result<DBusEvent>, StreamId, impl Stream<Item = DBusEvent>)> {
        let proxy = StatusNotifierItemProxy::builder(&conn)
            .destination(service.to_string())?
            .build()
            .await?;

        let event = proxy
            .icon_pixmap()
            .await
            .context("failed to get IconPixmap")
            .and_then(select_best_variant)
            .map(|(width, height, bytes)| DBusEvent::IconPixmapChanged {
                service: Arc::clone(&service),
                width,
                height,
                bytes,
            });

        let stream_id = StreamId::IconPixmapUpdated {
            service: Arc::clone(&service),
        };

        let stream = proxy
            .receive_icon_pixmap_changed()
            .await
            .filter_map(move |e| {
                let service = Arc::clone(&service);
                async move {
                    let variants = e.get().await.ok()?;
                    let (width, height, bytes) = select_best_variant(variants).ok()?;
                    Some(DBusEvent::IconPixmapChanged {
                        service: Arc::clone(&service),
                        width,
                        height,
                        bytes,
                    })
                }
            });

        Ok((event, stream_id, stream))
    }
}

fn select_best_variant(variants: Vec<(i32, i32, Vec<u8>)>) -> Result<(i32, i32, Vec<u8>)> {
    variants
        .into_iter()
        .max_by(|(w1, _, _), (w2, _, _)| w1.cmp(w2))
        .context("DBus returned IconPixmap but it has no variants")
}

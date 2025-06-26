use crate::{
    dbus_event::DBusEvent, status_notifier_item::proxy::StatusNotifierItemProxy,
    stream_id::StreamId,
};
use anyhow::{Context as _, Result};
use futures::{Stream, StreamExt};
use std::sync::Arc;
use zbus::Connection;

pub(crate) struct MenuUpdated;

impl MenuUpdated {
    pub(crate) async fn split(
        conn: Connection,
        service: Arc<str>,
    ) -> Result<(Result<DBusEvent>, StreamId, impl Stream<Item = DBusEvent>)> {
        let proxy = StatusNotifierItemProxy::builder(&conn)
            .destination(service.to_string())?
            .build()
            .await?;

        let event = proxy
            .menu()
            .await
            .context("failed to get Menu")
            .map(|menu| DBusEvent::MenuChanged {
                service: Arc::clone(&service),
                menu: Arc::new(menu),
            });

        let stream_id = StreamId::MenuUpdated {
            service: Arc::clone(&service),
        };

        let stream = proxy.receive_menu_changed().await.filter_map(move |e| {
            let service = Arc::clone(&service);
            async move {
                let menu = e.get().await.ok()?;
                Some(DBusEvent::MenuChanged {
                    service: Arc::clone(&service),
                    menu: Arc::new(menu),
                })
            }
        });

        Ok((event, stream_id, stream))
    }
}

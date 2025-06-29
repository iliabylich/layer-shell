use crate::{
    dbus_event::DBusEvent, status_notifier_item::proxy::StatusNotifierItemProxy,
    stream_id::StreamId,
};
use anyhow::{Context as _, Result, bail};
use futures::{Stream, StreamExt};
use std::sync::Arc;
use zbus::Connection;

pub(crate) struct IconNameUpdated;

impl IconNameUpdated {
    pub(crate) async fn get_current(conn: Connection, service: Arc<str>) -> Result<DBusEvent> {
        let proxy = StatusNotifierItemProxy::builder(&conn)
            .destination(service.to_string())?
            .build()
            .await?;

        let icon_name = proxy.icon_name().await.context("failed to get IconName")?;

        if icon_name.is_empty() {
            bail!("empty IconName, skipping");
        }

        Ok(DBusEvent::IconNameChanged {
            service: Arc::clone(&service),
            icon_name,
        })
    }

    pub(crate) async fn split(
        conn: Connection,
        service: Arc<str>,
    ) -> Result<(Result<DBusEvent>, StreamId, impl Stream<Item = DBusEvent>)> {
        let proxy = StatusNotifierItemProxy::builder(&conn)
            .destination(service.to_string())?
            .build()
            .await?;

        let event = Self::get_current(conn.clone(), Arc::clone(&service)).await;

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

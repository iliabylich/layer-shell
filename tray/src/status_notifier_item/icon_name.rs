use crate::{
    dbus_event::DBusEvent,
    stream_id::{ServiceStreamId, StreamId},
    tray_stream::TrayStream,
};
use anyhow::{Context as _, Result, bail};
use futures::{StreamExt, stream::BoxStream};
use std::sync::Arc;
use zbus::{Connection, proxy};

#[proxy(
    interface = "org.kde.StatusNotifierItem",
    default_path = "/StatusNotifierItem",
    assume_defaults = true
)]
trait StatusNotifierItem {
    #[zbus(property)]
    fn icon_name(&self) -> zbus::Result<String>;
}

pub(crate) struct IconName;

#[async_trait::async_trait]
impl TrayStream for IconName {
    type Input = Arc<str>;

    async fn stream(
        conn: &Connection,
        service: Self::Input,
    ) -> Result<(StreamId, BoxStream<'static, DBusEvent>)> {
        let id = StreamId::ServiceStream {
            service: Arc::clone(&service),
            id: ServiceStreamId::IconNameUpdated,
        };

        let proxy = StatusNotifierItemProxy::builder(&conn)
            .destination(service.to_string())?
            .build()
            .await?;

        let pre = match Self::get(conn, Arc::clone(&service)).await {
            Ok(icon_name) => {
                let event = DBusEvent::IconNameChanged {
                    service: Arc::clone(&service),
                    icon_name,
                };
                futures::stream::once(async move { event }).boxed()
            }
            Err(err) => {
                log::error!(target: "Tray", "{err:?}");
                futures::stream::empty().boxed()
            }
        };

        let post = proxy
            .receive_icon_name_changed()
            .await
            .filter_map(move |e| {
                let service = Arc::clone(&service);
                async move {
                    let icon_name = e.get().await.ok()?;
                    Some(DBusEvent::IconNameChanged { service, icon_name })
                }
            });

        Ok((id, pre.chain(post).boxed()))
    }
}

impl IconName {
    pub(crate) async fn get(conn: &Connection, service: Arc<str>) -> Result<String> {
        let proxy = StatusNotifierItemProxy::builder(conn)
            .destination(service.to_string())?
            .build()
            .await?;

        let icon_name = proxy.icon_name().await.context("failed to get IconName")?;

        if icon_name.is_empty() {
            bail!("empty IconName, skipping");
        }

        Ok(icon_name)
    }
}

use crate::{
    TrayIconPixmap,
    dbus_event::DBusEvent,
    stream_id::{ServiceStreamId, StreamId},
    tray_stream::TrayStream,
};
use anyhow::{Context as _, Result};
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
    fn icon_pixmap(&self) -> zbus::Result<Vec<(i32, i32, Vec<u8>)>>;
}

pub(crate) struct IconPixmap;

#[async_trait::async_trait]
impl TrayStream for IconPixmap {
    type Input = Arc<str>;

    async fn stream(
        conn: &Connection,
        service: Self::Input,
    ) -> Result<(StreamId, BoxStream<'static, DBusEvent>)> {
        let id = StreamId::ServiceStream {
            service: Arc::clone(&service),
            id: ServiceStreamId::IconPixmapUpdated,
        };

        let proxy = StatusNotifierItemProxy::builder(conn)
            .destination(service.to_string())?
            .build()
            .await?;

        let pre = match Self::get(conn, Arc::clone(&service)).await {
            Ok(pixmap) => {
                let event = DBusEvent::IconPixmapChanged {
                    service: Arc::clone(&service),
                    pixmap,
                };
                futures::stream::once(async move { event }).boxed()
            }
            Err(err) => {
                log::error!(target: "Tray", "{err:?}");
                futures::stream::empty().boxed()
            }
        };

        let post = proxy
            .receive_icon_pixmap_changed()
            .await
            .filter_map(move |e| {
                let service = Arc::clone(&service);
                async move {
                    let variants = e.get().await.ok()?;
                    let pixmap = select_best_variant(variants).ok()?;
                    Some(DBusEvent::IconPixmapChanged { service, pixmap })
                }
            });

        Ok((id, pre.chain(post).boxed()))
    }
}

impl IconPixmap {
    pub(crate) async fn get(conn: &Connection, service: Arc<str>) -> Result<TrayIconPixmap> {
        let proxy = StatusNotifierItemProxy::builder(conn)
            .destination(service.to_string())?
            .build()
            .await?;

        proxy
            .icon_pixmap()
            .await
            .context("failed to get IconPixmap")
            .and_then(select_best_variant)
    }
}

fn select_best_variant(variants: Vec<(i32, i32, Vec<u8>)>) -> Result<TrayIconPixmap> {
    let (width, height, bytes) = variants
        .into_iter()
        .max_by(|(w1, _, _), (w2, _, _)| w1.cmp(w2))
        .context("DBus returned IconPixmap but it has no variants")?;
    Ok(TrayIconPixmap {
        width,
        height,
        bytes: bytes.into(),
    })
}

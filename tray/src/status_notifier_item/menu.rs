use crate::{dbus_event::DBusEvent, stream_id::StreamId, tray_stream::TrayStream};
use anyhow::Result;
use futures::{StreamExt, stream::BoxStream};
use std::sync::Arc;
use zbus::{Connection, proxy, zvariant::OwnedObjectPath};

#[proxy(
    interface = "org.kde.StatusNotifierItem",
    default_path = "/StatusNotifierItem",
    assume_defaults = true
)]
trait StatusNotifierItem {
    #[zbus(property)]
    fn menu(&self) -> zbus::Result<OwnedObjectPath>;
}

pub(crate) struct Menu;

#[async_trait::async_trait]
impl TrayStream for Menu {
    type Input = Arc<str>;

    async fn stream(
        conn: &Connection,
        service: Self::Input,
    ) -> Result<(StreamId, BoxStream<'static, DBusEvent>)> {
        let id = StreamId::MenuUpdated {
            service: Arc::clone(&service),
        };

        let proxy = StatusNotifierItemProxy::builder(&conn)
            .destination(service.to_string())?
            .build()
            .await?;

        let pre = match proxy.menu().await {
            Ok(path) => {
                let event = DBusEvent::MenuChanged {
                    service: Arc::clone(&service),
                    menu: Arc::new(path),
                };
                futures::stream::once(async move { event }).boxed()
            }
            Err(err) => {
                log::error!(target: "Tray", "{err:?}");
                futures::stream::empty().boxed()
            }
        };

        let post = proxy
            .receive_menu_changed()
            .await
            .filter_map(move |e| {
                let service = Arc::clone(&service);
                async move {
                    let path = e.get().await.ok()?;
                    Some(DBusEvent::MenuChanged {
                        service: Arc::clone(&service),
                        menu: Arc::new(path),
                    })
                }
            })
            .boxed();

        Ok((id, pre.chain(post).boxed()))
    }
}

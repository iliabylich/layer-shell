use crate::{dbus_event::DBusEvent, stream_id::StreamId, tray_stream::TrayStream};
use anyhow::Result;
use futures::{StreamExt, stream::BoxStream};
use std::sync::Arc;
use zbus::Connection;

mod dbus {
    use zbus::proxy;

    #[proxy(
        interface = "org.kde.StatusNotifierItem",
        default_path = "/StatusNotifierItem",
        assume_defaults = true
    )]
    pub(crate) trait StatusNotifierItem {
        #[zbus(signal)]
        fn new_icon(&self) -> zbus::Result<()>;
    }
}

pub(crate) struct NewIcon;

#[async_trait::async_trait]
impl TrayStream for NewIcon {
    type Input = Arc<str>;

    async fn stream(
        conn: &Connection,
        service: Self::Input,
    ) -> Result<(StreamId, BoxStream<'static, DBusEvent>)> {
        let proxy = dbus::StatusNotifierItemProxy::builder(conn)
            .destination(service.to_string())?
            .build()
            .await?;

        let id = StreamId::NewIconNotified {
            service: Arc::clone(&service),
        };

        let stream = proxy
            .receive_new_icon()
            .await?
            .filter_map(move |_e| {
                let service = Arc::clone(&service);
                async move {
                    Some(DBusEvent::NewIconNotified {
                        service: Arc::clone(&service),
                    })
                }
            })
            .boxed();

        Ok((id, stream))
    }
}

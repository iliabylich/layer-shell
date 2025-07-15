use crate::{dbus_event::DBusEvent, stream_id::StreamId, tray_stream::TrayStream};
use anyhow::Result;
use futures::{StreamExt, stream::BoxStream};
use zbus::{Connection, fdo::DBusProxy};

pub(crate) struct NameLost;

#[async_trait::async_trait]
impl TrayStream for NameLost {
    type Input = ();

    async fn stream(
        conn: &Connection,
        _: Self::Input,
    ) -> Result<(StreamId, BoxStream<'static, DBusEvent>)> {
        let id = StreamId::NameLost;

        let dbus_proxy = DBusProxy::new(&conn).await?;
        let stream = dbus_proxy
            .receive_name_lost()
            .await?
            .filter_map(|e| async move {
                let args = e.args().ok()?;
                let name = args.name.to_string();
                Some(DBusEvent::NameLost(name))
            })
            .boxed();

        Ok((id, stream))
    }
}

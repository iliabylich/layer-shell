use crate::{dbus_event::DBusEvent, stream_id::StreamId, tray_stream::TrayStream};
use anyhow::Result;
use futures::{StreamExt, stream::BoxStream};
use zbus::{Connection, fdo::DBusProxy};

pub(crate) struct NameOwnerChangedEvent;

#[async_trait::async_trait]
impl TrayStream for NameOwnerChangedEvent {
    type Input = ();

    async fn stream(
        conn: &Connection,
        _: Self::Input,
    ) -> Result<(StreamId, BoxStream<'static, DBusEvent>)> {
        let id = StreamId::NameOwnedChanged;

        let dbus_proxy = DBusProxy::new(conn).await?;
        let stream = dbus_proxy
            .receive_name_owner_changed()
            .await?
            .filter_map(|e| async move {
                let args = e.args().ok()?;
                let name = args.name.to_string();
                let new_owner = args.new_owner.as_ref().map(|v| v.to_string());
                Some(DBusEvent::NameOwnerChanged { name, new_owner })
            })
            .boxed();

        Ok((id, stream))
    }
}

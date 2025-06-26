use crate::dbus_event::DBusEvent;
use anyhow::Result;
use futures::{Stream, StreamExt};
use zbus::{Connection, fdo::DBusProxy};

pub(crate) struct NameOwnerChangedEvent;

impl NameOwnerChangedEvent {
    pub(crate) async fn into_stream(conn: Connection) -> Result<impl Stream<Item = DBusEvent>> {
        let dbus_proxy = DBusProxy::new(&conn).await?;
        let stream = dbus_proxy
            .receive_name_owner_changed()
            .await?
            .filter_map(|e| async move {
                let args = e.args().ok()?;
                let name = args.name.to_string();
                let new_owner = args.new_owner.as_ref().map(|v| v.to_string());
                Some(DBusEvent::NameOwnerChanged { name, new_owner })
            });
        Ok(stream)
    }
}

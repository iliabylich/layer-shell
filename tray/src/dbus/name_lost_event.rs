use crate::dbus_event::DBusEvent;
use anyhow::Result;
use futures::{Stream, StreamExt};
use zbus::{Connection, fdo::DBusProxy};

pub(crate) struct NameLostEvent;

impl NameLostEvent {
    pub(crate) async fn into_stream(conn: Connection) -> Result<impl Stream<Item = DBusEvent>> {
        let dbus_proxy = DBusProxy::new(&conn).await?;
        let stream = dbus_proxy
            .receive_name_lost()
            .await?
            .filter_map(|e| async move {
                let args = e.args().ok()?;
                let name = args.name.to_string();
                Some(DBusEvent::NameLost(name))
            });
        Ok(stream)
    }
}

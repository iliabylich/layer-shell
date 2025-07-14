use crate::{dbus_event::DBusEvent, stream_id::StreamId};
use anyhow::Result;
use futures::stream::BoxStream;
use zbus::Connection;

#[async_trait::async_trait]
pub(crate) trait TrayStream {
    type Input;

    async fn stream(
        conn: &Connection,
        input: Self::Input,
    ) -> Result<(StreamId, BoxStream<'static, DBusEvent>)>;
}

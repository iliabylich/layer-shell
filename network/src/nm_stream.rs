use crate::{multiplexer::StreamId, nm_event::NetworkManagerEvent};
use anyhow::Result;
use futures::stream::BoxStream;
use zbus::Connection;

#[async_trait::async_trait]
pub(crate) trait NmStream {
    const ID: &StreamId;

    type Input;

    async fn stream(
        conn: &Connection,
        input: Self::Input,
    ) -> Result<BoxStream<'static, NetworkManagerEvent>>;
}

use async_stream::stream;
use futures::Stream;

mod event;
mod network_list;
mod wifi_status;

pub use event::{Event, Network, NetworkList, WiFiStatus};

pub fn connect() -> impl Stream<Item = Event> {
    stream! {
        let Ok((res, conn)) = dbus_tokio::connection::new_system_sync() else {
            log::error!("failed to connect to D-Bus");
            return;
        };

        tokio::spawn(async {
            let err = res.await;
            log::error!("Lost connection to D-Bus: {:?}", err);
        });

        loop {
            match network_list::get(conn.as_ref()).await {
                Ok(event) => yield event,
                Err(err) => log::error!("{:?}", err),
            }

            yield wifi_status::get(conn.as_ref()).await;

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }
}

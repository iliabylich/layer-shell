use crate::Event;
use std::sync::mpsc::Sender;

pub(crate) mod network_list;
pub(crate) mod wifi_status;

pub(crate) async fn spawn(tx: Sender<Event>) {
    let Ok((res, conn)) = dbus_tokio::connection::new_system_sync() else {
        log::error!("failed to connect to D-Bus");
        return;
    };

    tokio::spawn(async {
        let err = res.await;
        log::error!("Lost connection to D-Bus: {:?}", err);
    });

    loop {
        if let Err(err) = network_list::tick(&tx, conn.as_ref()).await {
            log::error!("{:?}", err);
        }
        if let Err(err) = wifi_status::tick(&tx, conn.as_ref()).await {
            log::error!("{:?}", err);
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

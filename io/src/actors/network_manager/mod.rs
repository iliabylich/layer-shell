use crate::Event;
use std::sync::{mpsc::Sender, Arc};

pub(crate) mod network_list;
pub(crate) mod wifi_status;

pub(crate) async fn spawn(tx: Sender<Event>) {
    let Ok((res, conn)) = dbus_tokio::connection::new_system_sync() else {
        log::error!("failed to connect to D-Bus");
        return;
    };

    tokio::spawn(async {
        let err = res.await;
        log::error!("Lost connection to D-Bus: {}", err);
    });

    tokio::join!(
        network_list::spawn(tx.clone(), Arc::clone(&conn)),
        wifi_status::spawn(tx.clone(), Arc::clone(&conn)),
    );
}

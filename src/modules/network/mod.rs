use crate::dbus::nm::NetworkManager;
use dbus::{blocking::Connection, Message};

mod network_list;
mod network_speed;
mod wifi_status;

pub(crate) fn setup() {
    let conn = match Connection::new_system() {
        Ok(conn) => conn,
        Err(err) => {
            log::error!("Failed to connect to D-Bus: {:?}", err);
            return;
        }
    };

    std::thread::spawn(move || {
        state_changed(&conn);

        let _id = NetworkManager::proxy(&conn).match_signal(
            move |_: crate::dbus::OrgFreedesktopNetworkManagerStateChanged,
                  conn: &Connection,
                  _: &Message| {
                state_changed(conn);
                true
            },
        );

        loop {
            if let Err(err) = conn.process(std::time::Duration::from_millis(1000)) {
                log::error!("D-Bus polling loop error: {:?}", err);
            }

            if let Err(err) = network_speed::update(&conn) {
                log::error!("{:?}", err);
            }
        }
    });
}

fn state_changed(conn: &Connection) {
    match network_list::get(conn) {
        Ok(event) => event.emit(),
        Err(err) => log::error!("{:?}", err),
    }

    let event = wifi_status::get(conn);
    event.emit();

    network_speed::reset();

    if let Ok(device) = NetworkManager::primary_wireless_device(conn) {
        if let Err(err) = network_speed::update(conn) {
            log::error!("{:?}", err);
        }

        if let Err(err) = device.set_refresh_rate_in_ms(conn, 1_000) {
            log::error!("{:?}", err);
        }
    }
}

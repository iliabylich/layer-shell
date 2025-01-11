use crate::global::global;
use dbus::{blocking::Connection, Message};

mod network_list;
mod wifi_status;

global!(CONNECTION, Connection);

pub(crate) fn setup() {
    std::thread::spawn(|| {
        match Connection::new_system() {
            Ok(conn) => CONNECTION::set(conn),
            Err(err) => {
                log::error!("Failed to connect to D-Bus: {:?}", err);
                return;
            }
        };

        reload();

        let proxy = CONNECTION::get().with_proxy(
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            std::time::Duration::from_millis(5000),
        );

        let _id = proxy.match_signal(
            |_: crate::dbus::OrgFreedesktopNetworkManagerStateChanged,
             _: &Connection,
             _: &Message| {
                reload();
                true
            },
        );

        loop {
            if let Err(err) = CONNECTION::get().process(std::time::Duration::from_millis(1000)) {
                log::error!("D-Bus polling loop error: {:?}", err);
            }
        }
    });
}

fn reload() {
    match network_list::get(CONNECTION::get()) {
        Ok(event) => event.emit(),
        Err(err) => log::error!("{:?}", err),
    }

    let event = wifi_status::get(CONNECTION::get());
    event.emit();
}

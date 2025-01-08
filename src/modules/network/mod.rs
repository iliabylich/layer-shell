use crate::global::global;

mod network_list;
mod wifi_status;

global!(CONNECTION, dbus::blocking::SyncConnection);

pub(crate) fn setup() {
    let conn = match dbus::blocking::SyncConnection::new_system() {
        Ok(conn) => conn,
        Err(err) => {
            log::error!("Failed to connect to D-Bus: {:?}", err);
            std::process::exit(1);
        }
    };

    CONNECTION::set(conn);
}

pub(crate) fn tick() {
    match network_list::get(CONNECTION::get()) {
        Ok(event) => event.emit(),
        Err(err) => log::error!("{:?}", err),
    }

    let event = wifi_status::get(CONNECTION::get());
    event.emit();
}

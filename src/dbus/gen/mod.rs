#[allow(dead_code)]
#[expect(clippy::bind_instead_of_map)]
pub(crate) mod nm;

#[allow(dead_code)]
pub(crate) mod nm_access_point;

#[allow(dead_code)]
pub(crate) mod nm_device;

#[allow(dead_code)]
#[expect(clippy::bind_instead_of_map)]
pub(crate) mod nm_device_wireless;

#[allow(dead_code)]
pub(crate) mod nm_ip4_config;

#[allow(dead_code)]
pub(crate) mod nm_active_connection;

#[allow(dead_code)]
pub(crate) mod nm_device_statistics;

#[allow(dead_code, unused_variables)]
pub(crate) mod status_notifier_watcher;

#[allow(dead_code)]
pub(crate) mod status_notifier_item;

#[allow(dead_code)]
#[expect(clippy::bind_instead_of_map)]
pub(crate) mod dbus_menu;

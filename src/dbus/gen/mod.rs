#[expect(dead_code)]
#[expect(clippy::bind_instead_of_map)]
pub(crate) mod nm;

#[expect(dead_code)]
pub(crate) mod nm_access_point;

#[expect(dead_code)]
pub(crate) mod nm_device;

#[expect(dead_code)]
#[expect(clippy::bind_instead_of_map)]
pub(crate) mod nm_device_wireless;

#[expect(dead_code)]
pub(crate) mod nm_ip4_config;

#[expect(dead_code)]
pub(crate) mod nm_active_connection;

#[expect(dead_code)]
pub(crate) mod nm_device_statistics;

#[expect(unused_variables)]
pub(crate) mod status_notifier_watcher;

#[expect(dead_code)]
pub(crate) mod status_notifier_item;

#[expect(dead_code)]
#[expect(clippy::bind_instead_of_map)]
pub(crate) mod dbus_menu;

pub(crate) mod layer_shell_control;

#[expect(clippy::bind_instead_of_map)]
pub(crate) mod pipewire_dbus;

#!/usr/bin/env bash

set -eu

getNetworkManagerClient() {
    local path="$1"
    local mod="$2"
    local base_url="https://raw.githubusercontent.com/NetworkManager/NetworkManager/refs/heads/main/introspection"
    local xml_url="$base_url/$path"
    wget "$xml_url" -O "src/dbus/interfaces/$path"
    dbus-codegen-rust --client blocking < "src/dbus/interfaces/$path" -o "src/dbus/gen/$mod"
    sed -i -e 's/pub/pub(crate)/g' "src/dbus/gen/$mod"
}

mkdir -p "src/dbus/interfaces"
mkdir -p "src/dbus/gen"

getNetworkManagerClient "org.freedesktop.NetworkManager.xml" "nm.rs"
getNetworkManagerClient "org.freedesktop.NetworkManager.Device.xml" "nm_device.rs"
getNetworkManagerClient "org.freedesktop.NetworkManager.IP4Config.xml" "nm_ip4_config.rs"
getNetworkManagerClient "org.freedesktop.NetworkManager.Device.Wireless.xml" "nm_device_wireless.rs"
getNetworkManagerClient "org.freedesktop.NetworkManager.AccessPoint.xml" "nm_access_point.rs"
getNetworkManagerClient "org.freedesktop.NetworkManager.AccessPoint.xml" "nm_access_point.rs"
getNetworkManagerClient "org.freedesktop.NetworkManager.Connection.Active.xml" "nm_active_connection.rs"
getNetworkManagerClient "org.freedesktop.NetworkManager.Device.Statistics.xml" "nm_device_statistics.rs"

dbus-codegen-rust --crossroads < dbus/org.kde.StatusNotifierWatcher.xml -o src/dbus/gen/status_notifier_watcher.rs
sed -i -e 's/pub/pub(crate)/g' src/dbus/gen/status_notifier_watcher.rs
sed -i -e 's/_, t/ctx, t/g' src/dbus/gen/status_notifier_watcher.rs
sed -i -e 's/register_status_notifier_item(service/register_status_notifier_item(service, ctx/g' src/dbus/gen/status_notifier_watcher.rs
sed -i -e 's/register_status_notifier_item(&mut self, service: String)/register_status_notifier_item(\&mut self, service: String, ctx: \&dbus_crossroads::Context)/' src/dbus/gen/status_notifier_watcher.rs

dbus-codegen-rust --client blocking < dbus/org.kde.StatusNotifierItem.xml -o src/dbus/gen/status_notifier_item.rs
sed -i -e 's/pub/pub(crate)/g' src/dbus/gen/status_notifier_item.rs

dbus-codegen-rust --client blocking < dbus/com.canonical.dbusmenu.xml -o src/dbus/gen/dbus_menu.rs
sed -i -e 's/pub/pub(crate)/g' src/dbus/gen/dbus_menu.rs

dbus-codegen-rust --crossroads < dbus/org.me.LayerShellControl.xml -o src/dbus/gen/layer_shell_control.rs
sed -i -e 's/pub/pub(crate)/g' src/dbus/gen/layer_shell_control.rs

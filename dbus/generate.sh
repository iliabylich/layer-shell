#!/usr/bin/env bash

set -eu

getXML() {
    local path="$1"
    local base_url="https://raw.githubusercontent.com/NetworkManager/NetworkManager/refs/heads/main/introspection"
    local xml_url="$base_url/$path"
    wget "$xml_url" -O "src/dbus/interfaces/$path"
}

genXML() {
    local path="$1"
    local mod="$2"
    dbus-codegen-rust --client blocking < "src/dbus/interfaces/$path" -o "src/dbus/gen/$2"
    sed -i -e 's/pub/pub(crate)/g' "src/dbus/gen/$2"
}

processXML() {
    local path="$1"
    local mod="$2"
    getXML "$path"
    genXML "$path" "$mod"
}

mkdir -p "src/dbus/interfaces"
mkdir -p "src/dbus/gen"

processXML "org.freedesktop.NetworkManager.xml" "nm.rs"
processXML "org.freedesktop.NetworkManager.Device.xml" "nm_device.rs"
processXML "org.freedesktop.NetworkManager.IP4Config.xml" "nm_ip4_config.rs"
processXML "org.freedesktop.NetworkManager.Device.Wireless.xml" "nm_device_wireless.rs"
processXML "org.freedesktop.NetworkManager.AccessPoint.xml" "nm_access_point.rs"
processXML "org.freedesktop.NetworkManager.AccessPoint.xml" "nm_access_point.rs"
processXML "org.freedesktop.NetworkManager.Connection.Active.xml" "nm_active_connection.rs"
processXML "org.freedesktop.NetworkManager.Device.Statistics.xml" "nm_device_statistics.rs"

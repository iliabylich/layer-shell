// This code was autogenerated with `dbus-codegen-rust --client blocking -o src/dbus/gen/nm_device_wireless.rs`, see https://github.com/diwic/dbus-rs
use dbus as dbus;
#[allow(unused_imports)]
use dbus::arg;
use dbus::blocking;

pub trait OrgFreedesktopNetworkManagerDeviceWireless {
    fn get_access_points(&self) -> Result<Vec<dbus::Path<'static>>, dbus::Error>;
    fn get_all_access_points(&self) -> Result<Vec<dbus::Path<'static>>, dbus::Error>;
    fn request_scan(&self, options: arg::PropMap) -> Result<(), dbus::Error>;
    fn hw_address(&self) -> Result<String, dbus::Error>;
    fn perm_hw_address(&self) -> Result<String, dbus::Error>;
    fn mode(&self) -> Result<u32, dbus::Error>;
    fn bitrate(&self) -> Result<u32, dbus::Error>;
    fn access_points(&self) -> Result<Vec<dbus::Path<'static>>, dbus::Error>;
    fn active_access_point(&self) -> Result<dbus::Path<'static>, dbus::Error>;
    fn wireless_capabilities(&self) -> Result<u32, dbus::Error>;
    fn last_scan(&self) -> Result<i64, dbus::Error>;
}

#[derive(Debug)]
pub struct OrgFreedesktopNetworkManagerDeviceWirelessAccessPointAdded {
    pub access_point: dbus::Path<'static>,
}

impl arg::AppendAll for OrgFreedesktopNetworkManagerDeviceWirelessAccessPointAdded {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.access_point, i);
    }
}

impl arg::ReadAll for OrgFreedesktopNetworkManagerDeviceWirelessAccessPointAdded {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgFreedesktopNetworkManagerDeviceWirelessAccessPointAdded {
            access_point: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for OrgFreedesktopNetworkManagerDeviceWirelessAccessPointAdded {
    const NAME: &'static str = "AccessPointAdded";
    const INTERFACE: &'static str = "org.freedesktop.NetworkManager.Device.Wireless";
}

#[derive(Debug)]
pub struct OrgFreedesktopNetworkManagerDeviceWirelessAccessPointRemoved {
    pub access_point: dbus::Path<'static>,
}

impl arg::AppendAll for OrgFreedesktopNetworkManagerDeviceWirelessAccessPointRemoved {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.access_point, i);
    }
}

impl arg::ReadAll for OrgFreedesktopNetworkManagerDeviceWirelessAccessPointRemoved {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgFreedesktopNetworkManagerDeviceWirelessAccessPointRemoved {
            access_point: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for OrgFreedesktopNetworkManagerDeviceWirelessAccessPointRemoved {
    const NAME: &'static str = "AccessPointRemoved";
    const INTERFACE: &'static str = "org.freedesktop.NetworkManager.Device.Wireless";
}

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target=T>> OrgFreedesktopNetworkManagerDeviceWireless for blocking::Proxy<'a, C> {

    fn get_access_points(&self) -> Result<Vec<dbus::Path<'static>>, dbus::Error> {
        self.method_call("org.freedesktop.NetworkManager.Device.Wireless", "GetAccessPoints", ())
            .and_then(|r: (Vec<dbus::Path<'static>>, )| Ok(r.0, ))
    }

    fn get_all_access_points(&self) -> Result<Vec<dbus::Path<'static>>, dbus::Error> {
        self.method_call("org.freedesktop.NetworkManager.Device.Wireless", "GetAllAccessPoints", ())
            .and_then(|r: (Vec<dbus::Path<'static>>, )| Ok(r.0, ))
    }

    fn request_scan(&self, options: arg::PropMap) -> Result<(), dbus::Error> {
        self.method_call("org.freedesktop.NetworkManager.Device.Wireless", "RequestScan", (options, ))
    }

    fn hw_address(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Device.Wireless", "HwAddress")
    }

    fn perm_hw_address(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Device.Wireless", "PermHwAddress")
    }

    fn mode(&self) -> Result<u32, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Device.Wireless", "Mode")
    }

    fn bitrate(&self) -> Result<u32, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Device.Wireless", "Bitrate")
    }

    fn access_points(&self) -> Result<Vec<dbus::Path<'static>>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Device.Wireless", "AccessPoints")
    }

    fn active_access_point(&self) -> Result<dbus::Path<'static>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Device.Wireless", "ActiveAccessPoint")
    }

    fn wireless_capabilities(&self) -> Result<u32, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Device.Wireless", "WirelessCapabilities")
    }

    fn last_scan(&self) -> Result<i64, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Device.Wireless", "LastScan")
    }
}

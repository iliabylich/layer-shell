// This code was autogenerated with `dbus-codegen-rust --client blocking -o src/dbus/gen/nm_active_connection.rs`, see https://github.com/diwic/dbus-rs
use dbus as dbus;
#[allow(unused_imports)]
use dbus::arg;
use dbus::blocking;

pub(crate) trait OrgFreedesktopNetworkManagerConnectionActive {
    fn connection(&self) -> Result<dbus::Path<'static>, dbus::Error>;
    fn specific_object(&self) -> Result<dbus::Path<'static>, dbus::Error>;
    fn id(&self) -> Result<String, dbus::Error>;
    fn uuid(&self) -> Result<String, dbus::Error>;
    fn type_(&self) -> Result<String, dbus::Error>;
    fn devices(&self) -> Result<Vec<dbus::Path<'static>>, dbus::Error>;
    fn state(&self) -> Result<u32, dbus::Error>;
    fn state_flags(&self) -> Result<u32, dbus::Error>;
    fn default(&self) -> Result<bool, dbus::Error>;
    fn ip4_config(&self) -> Result<dbus::Path<'static>, dbus::Error>;
    fn dhcp4_config(&self) -> Result<dbus::Path<'static>, dbus::Error>;
    fn default6(&self) -> Result<bool, dbus::Error>;
    fn ip6_config(&self) -> Result<dbus::Path<'static>, dbus::Error>;
    fn dhcp6_config(&self) -> Result<dbus::Path<'static>, dbus::Error>;
    fn vpn(&self) -> Result<bool, dbus::Error>;
    fn controller(&self) -> Result<dbus::Path<'static>, dbus::Error>;
    fn master(&self) -> Result<dbus::Path<'static>, dbus::Error>;
}

#[derive(Debug)]
pub(crate) struct OrgFreedesktopNetworkManagerConnectionActiveStateChanged {
    pub(crate) state: u32,
    pub(crate) reason: u32,
}

impl arg::AppendAll for OrgFreedesktopNetworkManagerConnectionActiveStateChanged {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.state, i);
        arg::RefArg::append(&self.reason, i);
    }
}

impl arg::ReadAll for OrgFreedesktopNetworkManagerConnectionActiveStateChanged {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgFreedesktopNetworkManagerConnectionActiveStateChanged {
            state: i.read()?,
            reason: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for OrgFreedesktopNetworkManagerConnectionActiveStateChanged {
    const NAME: &'static str = "StateChanged";
    const INTERFACE: &'static str = "org.freedesktop.NetworkManager.Connection.Active";
}

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target=T>> OrgFreedesktopNetworkManagerConnectionActive for blocking::Proxy<'a, C> {

    fn connection(&self) -> Result<dbus::Path<'static>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Connection")
    }

    fn specific_object(&self) -> Result<dbus::Path<'static>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "SpecificObject")
    }

    fn id(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Id")
    }

    fn uuid(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Uuid")
    }

    fn type_(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Type")
    }

    fn devices(&self) -> Result<Vec<dbus::Path<'static>>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Devices")
    }

    fn state(&self) -> Result<u32, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "State")
    }

    fn state_flags(&self) -> Result<u32, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "StateFlags")
    }

    fn default(&self) -> Result<bool, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Default")
    }

    fn ip4_config(&self) -> Result<dbus::Path<'static>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Ip4Config")
    }

    fn dhcp4_config(&self) -> Result<dbus::Path<'static>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Dhcp4Config")
    }

    fn default6(&self) -> Result<bool, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Default6")
    }

    fn ip6_config(&self) -> Result<dbus::Path<'static>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Ip6Config")
    }

    fn dhcp6_config(&self) -> Result<dbus::Path<'static>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Dhcp6Config")
    }

    fn vpn(&self) -> Result<bool, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Vpn")
    }

    fn controller(&self) -> Result<dbus::Path<'static>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Controller")
    }

    fn master(&self) -> Result<dbus::Path<'static>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.freedesktop.NetworkManager.Connection.Active", "Master")
    }
}

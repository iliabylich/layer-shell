#[expect(clippy::needless_lifetimes)]
mod generated;

pub(crate) mod nm;
pub(crate) mod tray;

pub(crate) use generated::nm::OrgFreedesktopNetworkManagerStateChanged;

pub(crate) use generated::status_notifier_watcher::{
    OrgKdeStatusNotifierWatcher, register_org_kde_status_notifier_watcher,
};

pub(crate) use generated::status_notifier_item::{
    OrgKdeStatusNotifierItemNewAttentionIcon, OrgKdeStatusNotifierItemNewIcon,
    OrgKdeStatusNotifierItemNewOverlayIcon, OrgKdeStatusNotifierItemNewStatus,
    OrgKdeStatusNotifierItemNewTitle, OrgKdeStatusNotifierItemNewToolTip,
};

pub(crate) use generated::dbus_menu::{
    ComCanonicalDbusmenuItemsPropertiesUpdated, ComCanonicalDbusmenuLayoutUpdated,
};

pub(crate) use generated::layer_shell_control::{
    OrgMeLayerShellControl, register_org_me_layer_shell_control,
};

pub(crate) use generated::pipewire_dbus::{OrgLocalPipewireDBus, OrgLocalPipewireDBusDataChanged};

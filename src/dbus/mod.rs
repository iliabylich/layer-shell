#[expect(clippy::needless_lifetimes)]
mod gen;

pub(crate) mod nm;
pub(crate) mod tray;

pub(crate) use gen::nm::OrgFreedesktopNetworkManagerStateChanged;

pub(crate) use gen::status_notifier_watcher::{
    register_org_kde_status_notifier_watcher, OrgKdeStatusNotifierWatcher,
};

pub(crate) use gen::status_notifier_item::{
    OrgKdeStatusNotifierItemNewAttentionIcon, OrgKdeStatusNotifierItemNewIcon,
    OrgKdeStatusNotifierItemNewOverlayIcon, OrgKdeStatusNotifierItemNewStatus,
    OrgKdeStatusNotifierItemNewTitle, OrgKdeStatusNotifierItemNewToolTip,
};

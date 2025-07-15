use crate::{TrayIconPixmap, TrayItem};
use std::sync::Arc;
use zbus::zvariant::OwnedObjectPath;

pub(crate) enum DBusEvent {
    // Global DBus
    NameLost(String),
    NameOwnerChanged {
        name: String,
        new_owner: Option<String>,
    },

    // ServiceNotifierWatcher
    ServiceAdded(Arc<str>),

    // ServiceNotifierItem
    IconNameChanged {
        service: Arc<str>,
        icon_name: String,
    },
    IconPixmapChanged {
        service: Arc<str>,
        pixmap: TrayIconPixmap,
    },
    MenuChanged {
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    },
    NewIconReceived {
        service: Arc<str>,
    },

    // DBusMenu
    LayoutUpdated {
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    },
    ItemsPropertiesUpdated {
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    },

    // custom
    LayoutReceived {
        service: Arc<str>,
        items: Vec<TrayItem>,
    },
}

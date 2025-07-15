use crate::{
    TrayAppAddedEvent, TrayAppIconUpdatedEvent, TrayAppMenuUpdatedEvent, TrayEvent, TrayIcon,
    TrayItem,
};
use std::sync::Arc;

pub(crate) enum Diff {
    Added {
        icon: TrayIcon,
        items: Vec<TrayItem>,
    },
    IconUpdated(TrayIcon),
    ItemsUpdated(Vec<TrayItem>),
    StillIncomplete,
}

impl Diff {
    pub(crate) fn into_event(self, service: Arc<str>) -> Option<TrayEvent> {
        match self {
            Self::Added { icon, items } => Some(TrayEvent::AppAdded(TrayAppAddedEvent {
                service: service.into(),
                items: items.into(),
                icon,
            })),
            Self::IconUpdated(icon) => Some(TrayEvent::AppIconUpdated(TrayAppIconUpdatedEvent {
                service: service.into(),
                icon,
            })),
            Self::ItemsUpdated(items) => Some(TrayEvent::AppMenuUpdated(TrayAppMenuUpdatedEvent {
                service: service.into(),
                items: items.into(),
            })),
            Self::StillIncomplete => None,
        }
    }
}

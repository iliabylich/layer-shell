use std::sync::Arc;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum StreamId {
    NameLost,
    NameOwnedChanged,

    Manual,
    ServiceAdded,

    IconNameUpdated { service: Arc<str> },
    IconPixmapUpdated { service: Arc<str> },
    MenuUpdated { service: Arc<str> },
    LayoutUpdated { service: Arc<str> },
    ItemsPropertiesUpdated { service: Arc<str> },
}

impl StreamId {
    fn service(&self) -> Option<&str> {
        match self {
            StreamId::NameLost
            | StreamId::NameOwnedChanged
            | StreamId::Manual
            | StreamId::ServiceAdded => None,

            StreamId::IconNameUpdated { service }
            | StreamId::IconPixmapUpdated { service }
            | StreamId::MenuUpdated { service }
            | StreamId::LayoutUpdated { service }
            | StreamId::ItemsPropertiesUpdated { service } => Some(service.as_ref()),
        }
    }

    pub(crate) fn is_related_to(&self, service: &str) -> bool {
        self.service().is_some_and(|v| v == service)
    }
}

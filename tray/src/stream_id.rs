use std::sync::Arc;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum StreamId {
    Manual,

    NameLost,
    NameOwnedChanged,
    ServiceAdded,

    ServiceStream {
        service: Arc<str>,
        id: ServiceStreamId,
    },
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum ServiceStreamId {
    IconNameUpdated,
    IconPixmapUpdated,
    MenuUpdated,
    LayoutUpdated,
    ItemsPropertiesUpdated,
    NewIconReceived,
}

impl StreamId {
    pub(crate) fn is_related_to_service(&self, pattern: &str) -> bool {
        if let Self::ServiceStream { service, .. } = self {
            service.as_ref() == pattern
        } else {
            false
        }
    }
}

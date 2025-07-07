use crate::{
    TrayAppAddedEvent, TrayAppIconUpdatedEvent, TrayAppMenuUpdatedEvent, TrayAppRemovedEvent,
    TrayEvent, TrayIcon, TrayItem,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Default)]
pub(crate) struct Store {
    icons: HashMap<Arc<str>, TrayIcon>,
    items: HashMap<Arc<str>, Vec<TrayItem>>,
}

impl Store {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn get_icon_and_items<'a>(
        &'a self,
        service: &Arc<str>,
    ) -> Option<(&'a TrayIcon, &'a Vec<TrayItem>)> {
        let icon = self.icons.get(service)?;
        let items = self.items.get(service)?;
        Some((icon, items))
    }

    pub(crate) fn update_icon(&mut self, service: Arc<str>, icon: TrayIcon) -> Option<TrayEvent> {
        let had_icon_before = self.icons.insert(Arc::clone(&service), icon).is_some();
        let (icon, items) = self.get_icon_and_items(&service)?;
        log::info!("had_icon_before = {had_icon_before}");

        let service = service.to_string().into();
        let icon = icon.clone();

        if had_icon_before {
            Some(TrayEvent::AppIconUpdated(TrayAppIconUpdatedEvent {
                service,
                icon,
            }))
        } else {
            Some(TrayEvent::AppAdded(TrayAppAddedEvent {
                service,
                items: items.clone().into(),
                icon,
            }))
        }
    }

    pub(crate) fn update_item(
        &mut self,
        service: Arc<str>,
        items: Vec<TrayItem>,
    ) -> Option<TrayEvent> {
        let had_items_before = self.items.insert(Arc::clone(&service), items).is_some();
        let (icon, items) = self.get_icon_and_items(&service)?;
        log::info!("had_items_before = {had_items_before}");

        let service = service.to_string().into();
        let items = items.clone().into();

        if had_items_before {
            Some(TrayEvent::AppMenuUpdated(TrayAppMenuUpdatedEvent {
                service,
                items,
            }))
        } else {
            Some(TrayEvent::AppAdded(TrayAppAddedEvent {
                service,
                items,
                icon: icon.clone(),
            }))
        }
    }

    pub(crate) fn remove(&mut self, service: Arc<str>) -> Option<TrayEvent> {
        let removed =
            self.icons.remove(&service).is_some() || self.items.remove(&service).is_some();

        if removed {
            Some(TrayEvent::AppRemoved(TrayAppRemovedEvent {
                service: service.to_string().into(),
            }))
        } else {
            None
        }
    }
}

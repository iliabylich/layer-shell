use crate::{TrayIcon, TrayItem, store::diff::Diff};

#[derive(Default)]
pub(crate) struct Data {
    icon: Option<TrayIcon>,
    items: Option<Vec<TrayItem>>,
}

impl Data {
    pub(crate) fn is_full(&self) -> bool {
        self.icon.is_some() && self.items.is_some()
    }

    pub(crate) fn set_icon(&mut self, icon: TrayIcon) -> Diff {
        let Some(items) = self.items.as_ref() else {
            self.icon = Some(icon);
            return Diff::StillIncomplete;
        };

        let had_icon = self.icon.is_some();
        self.icon = Some(icon.clone());
        if had_icon {
            Diff::IconUpdated(icon)
        } else {
            Diff::Added {
                icon,
                items: items.clone(),
            }
        }
    }

    pub(crate) fn set_items(&mut self, items: Vec<TrayItem>) -> Diff {
        let Some(icon) = self.icon.as_ref() else {
            self.items = Some(items);
            return Diff::StillIncomplete;
        };

        let had_items = self.items.is_some();
        self.items = Some(items.clone());
        if had_items {
            Diff::ItemsUpdated(items)
        } else {
            Diff::Added {
                icon: icon.clone(),
                items,
            }
        }
    }
}

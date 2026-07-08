use crate::{
    modules::{TrayItem, tray::uuid::UUID},
    utils::{StringRef, StringRefExt as _},
};
use alloc::{vec, vec::Vec};
use dbus::messages::sni_host::{GetLayoutItem, GetLayoutList};

/// cbindgen:ignore
pub(crate) type GetLayout =
    dbus::messages::sni_host::GetLayout<VecOfTrayItems, TrayItem, StringRef>;

#[derive(Default)]
pub struct VecOfTrayItems(pub(crate) Vec<TrayItem>);

impl GetLayoutList for VecOfTrayItems {
    type Item = TrayItem;

    fn new() -> Self {
        Self(vec![])
    }

    fn push(&mut self, item: Self::Item) {
        self.0.push(item);
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl GetLayoutItem for TrayItem {
    type List = VecOfTrayItems;

    fn new_section(children: Self::List) -> Self {
        Self::Section {
            children: children.0.into(),
        }
    }

    fn new_nested(id: i32, service: &str, label: &str, children: Self::List) -> Self {
        Self::Nested {
            id,
            uuid: UUID::encode(service, id),
            label: StringRef::new(label),
            children: children.0.into(),
        }
    }

    fn new_disabled(id: i32, service: &str, label: &str) -> Self {
        Self::Disabled {
            id,
            uuid: UUID::encode(service, id),
            label: StringRef::new(label),
        }
    }

    fn new_checkbox(id: i32, service: &str, label: &str, checked: bool) -> Self {
        Self::Checkbox {
            id,
            uuid: UUID::encode(service, id),
            label: StringRef::new(label),
            checked,
        }
    }

    fn new_radio(id: i32, service: &str, label: &str, selected: bool) -> Self {
        Self::Radio {
            id,
            uuid: UUID::encode(service, id),
            label: StringRef::new(label),
            selected,
        }
    }

    fn new_regular(id: i32, service: &str, label: &str) -> Self {
        Self::Regular {
            id,
            uuid: UUID::encode(service, id),
            label: StringRef::new(label),
        }
    }
}

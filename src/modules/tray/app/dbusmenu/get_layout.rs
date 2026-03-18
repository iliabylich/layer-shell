use crate::{
    dbus::{
        Message, OneshotResource,
        decoder::{ArrayValue, Body, Value},
        messages::value_is,
    },
    ffi::ShortString,
    modules::{TrayItem, tray::uuid::UUID},
};
use anyhow::{Context, Result};
use std::borrow::Cow;

pub(crate) struct GetLayout {
    service: ShortString,
    menu: String,
}

impl GetLayout {
    pub(crate) fn new(service: ShortString) -> Self {
        Self {
            service,
            menu: String::new(),
        }
    }
}

impl OneshotResource for GetLayout {
    type Input = (ShortString, ShortString);
    type Output = Vec<TrayItem>;

    fn make_request(&self, (destination, path): (ShortString, ShortString)) -> Message<'static> {
        use crate::dbus::types::{CompleteType, Value};

        let body = vec![
            Value::Int32(0),
            Value::Int32(1),
            Value::Array(
                CompleteType::String,
                vec![
                    Value::String(Cow::Borrowed("type")),
                    Value::String(Cow::Borrowed("label")),
                    Value::String(Cow::Borrowed("enabled")),
                    Value::String(Cow::Borrowed("visible")),
                    Value::String(Cow::Borrowed("icon-name")),
                    Value::String(Cow::Borrowed("icon-data")),
                    Value::String(Cow::Borrowed("shortcut")),
                    Value::String(Cow::Borrowed("toggle-type")),
                    Value::String(Cow::Borrowed("toggle-state")),
                    Value::String(Cow::Borrowed("children-display")),
                ],
            ),
        ];

        Message::MethodCall {
            destination: Some(Cow::Owned(destination.to_string())),
            path: Cow::Owned(path.to_string()),
            interface: Some(Cow::Borrowed("com.canonical.dbusmenu")),
            serial: 0,
            member: Cow::Borrowed("GetLayout"),
            sender: None,
            unix_fds: None,
            body,
        }
    }

    fn try_process(&self, mut body: Body<'_>) -> Result<Self::Output> {
        let _ = body.try_next()?.context("no root item id")?;
        let root = body.try_next()?.context("no root")?;
        value_is!(root, Value::Struct(root));

        let mut iter = root.iter()?;
        let _ = iter.try_next()?.context("expected 3 items")?;
        let _ = iter.try_next()?.context("expected 3 items")?;
        let top_level_items = iter.try_next()?.context("expected 3 items")?;

        value_is!(top_level_items, Value::Array(top_level_items));

        parse_items(self.service, &self.menu, top_level_items)
    }
}

fn parse_items(service: ShortString, menu: &str, items: ArrayValue<'_>) -> Result<Vec<TrayItem>> {
    let mut out = vec![];
    let mut batch = vec![];
    let mut iter = items.iter();

    while let Some(item) = iter.try_next()? {
        value_is!(item, Value::Variant(item));
        let item = item.materialize()?;
        let item = parse_item(service, menu, item)?;
        match item {
            ItemOrSeparator::Skip => continue,
            ItemOrSeparator::Item(item) => batch.push(item),
            ItemOrSeparator::Separator => {
                let section = TrayItem::Section {
                    children: std::mem::take(&mut batch).into(),
                };
                out.push(section);
            }
        }
    }

    if !batch.is_empty() {
        if out.is_empty() {
            out = batch;
        } else {
            let section = TrayItem::Section {
                children: std::mem::take(&mut batch).into(),
            };
            out.push(section);
        }
    }

    Ok(out)
}

fn parse_item(service: ShortString, menu: &str, item: Value<'_>) -> Result<ItemOrSeparator> {
    value_is!(item, Value::Struct(fields));

    let mut fields_iter = fields.iter()?;

    let id = fields_iter.try_next()?.context("expected 3 items")?;
    value_is!(id, Value::Int32(id));
    let uuid = UUID::encode(service, id);

    let props = fields_iter.try_next()?.context("expected 3 items")?;
    value_is!(props, Value::Array(props));
    let mut props_iter = props.iter();
    let mut type_ = "standard";
    let mut label = "";
    let mut enabled = true;
    let mut visible = true;
    let mut toggle_type = "";
    let mut toggle_state = -1;
    let mut children_display = "";
    while let Some(prop) = props_iter.try_next()? {
        value_is!(prop, Value::DictEntry(dict_entry));
        let (key, value) = dict_entry.key_value()?;
        value_is!(key, Value::String(key));
        value_is!(value, Value::Variant(value));

        match key {
            "type" => {
                let value = value.materialize()?;
                value_is!(value, Value::String(value));
                type_ = value;
            }
            "label" => {
                let value = value.materialize()?;
                value_is!(value, Value::String(value));
                label = value;
            }
            "enabled" => {
                let value = value.materialize()?;
                value_is!(value, Value::Bool(value));
                enabled = value;
            }
            "visible" => {
                let value = value.materialize()?;
                value_is!(value, Value::Bool(value));
                visible = value;
            }
            "toggle-type" => {
                let value = value.materialize()?;
                value_is!(value, Value::String(value));
                toggle_type = value;
            }
            "toggle-state" => {
                let value = value.materialize()?;
                value_is!(value, Value::Int32(value));
                toggle_state = value;
            }
            "children-display" => {
                let value = value.materialize()?;
                value_is!(value, Value::String(value));
                children_display = value;
            }

            _ => log::warn!(target: "Tray", "Unknown dbusmenu prop: {key}"),
        }
    }

    let children_values = fields_iter.try_next()?.context("expected 3 items")?;
    value_is!(children_values, Value::Array(children_values));
    let children = parse_items(service, menu, children_values)?;

    if !visible {
        Ok(ItemOrSeparator::Skip)
    } else if children_display == "submenu" {
        Ok(ItemOrSeparator::Item(TrayItem::Nested {
            id,
            uuid,
            label: ShortString::from(label),
            children: children.into(),
        }))
    } else if type_ == "separator" {
        Ok(ItemOrSeparator::Separator)
    } else if !enabled {
        Ok(ItemOrSeparator::Item(TrayItem::Disabled {
            id,
            uuid,
            label: ShortString::from(label),
        }))
    } else if toggle_type == "checkmark" {
        Ok(ItemOrSeparator::Item(TrayItem::Checkbox {
            id,
            uuid,
            label: ShortString::from(label),
            checked: toggle_state == 1,
        }))
    } else if toggle_type == "radio" {
        Ok(ItemOrSeparator::Item(TrayItem::Radio {
            id,
            uuid,
            label: ShortString::from(label),
            selected: toggle_state == 1,
        }))
    } else {
        Ok(ItemOrSeparator::Item(TrayItem::Regular {
            id,
            uuid,
            label: ShortString::from(label),
        }))
    }
}

#[derive(Debug)]
enum ItemOrSeparator {
    Item(TrayItem),
    Separator,
    Skip,
}

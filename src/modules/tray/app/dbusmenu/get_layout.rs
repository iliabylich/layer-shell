use crate::{
    dbus::{
        MethodCall, OutgoingMessage,
        decoder::{ArrayValue, Value},
        messages::value_is,
    },
    modules::{TrayItem, tray::uuid::UUID},
    sansio::DBusConnectionKind,
    utils::StringRef,
};
use anyhow::{Context, Result};

pub(crate) const GET_LAYOUT: MethodCall<(StringRef, StringRef), Vec<TrayItem>, StringRef> =
    MethodCall::builder()
        .send(&|(destination, path), _service| {
            use crate::dbus::types::{CompleteType, Value};

            OutgoingMessage::MethodCall {
                destination: Some(destination),
                path,
                interface: Some(StringRef::new("com.canonical.dbusmenu")),
                serial: 0,
                member: StringRef::new("GetLayout"),
                sender: None,
                unix_fds: None,
                body: vec![
                    Value::Int32(0),
                    Value::Int32(1),
                    Value::Array(
                        CompleteType::String,
                        vec![
                            Value::StringRef(StringRef::new("type")),
                            Value::StringRef(StringRef::new("label")),
                            Value::StringRef(StringRef::new("enabled")),
                            Value::StringRef(StringRef::new("visible")),
                            Value::StringRef(StringRef::new("icon-name")),
                            Value::StringRef(StringRef::new("icon-data")),
                            Value::StringRef(StringRef::new("shortcut")),
                            Value::StringRef(StringRef::new("toggle-type")),
                            Value::StringRef(StringRef::new("toggle-state")),
                            Value::StringRef(StringRef::new("children-display")),
                        ],
                    ),
                ],
            }
        })
        .try_process(&|mut body, service| {
            let _ = body.try_next()?.context("no root item id")?;
            let root = body.try_next()?.context("no root")?;
            value_is!(root, Value::Struct(root));

            let mut iter = root.iter()?;
            let _ = iter.try_next()?.context("expected 3 items")?;
            let _ = iter.try_next()?.context("expected 3 items")?;
            let top_level_items = iter.try_next()?.context("expected 3 items")?;

            value_is!(top_level_items, Value::Array(top_level_items));

            parse_items(service, top_level_items)
        })
        .kind(DBusConnectionKind::Session);

fn parse_items(service: StringRef, items: ArrayValue<'_>) -> Result<Vec<TrayItem>> {
    let mut out = vec![];
    let mut batch = vec![];
    let mut iter = items.iter();

    while let Some(item) = iter.try_next()? {
        value_is!(item, Value::Variant(item));
        let item = item.materialize()?;
        let item = parse_item(service.clone(), item)?;
        match item {
            ItemOrSeparator::Skip => continue,
            ItemOrSeparator::Item(item) => batch.push(item),
            ItemOrSeparator::Separator => {
                let section = TrayItem::Section {
                    children: core::mem::take(&mut batch).into(),
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
                children: core::mem::take(&mut batch).into(),
            };
            out.push(section);
        }
    }

    Ok(out)
}

fn parse_item(service: StringRef, item: Value<'_>) -> Result<ItemOrSeparator> {
    value_is!(item, Value::Struct(fields));

    let mut fields_iter = fields.iter()?;

    let id = fields_iter.try_next()?.context("expected 3 items")?;
    value_is!(id, Value::Int32(id));
    let uuid = UUID::encode(service.clone(), id);

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
    let children = parse_items(service, children_values)?;

    if label.len() > 100 {
        label = &label[..100];
    }

    if !visible {
        Ok(ItemOrSeparator::Skip)
    } else if children_display == "submenu" {
        Ok(ItemOrSeparator::Item(TrayItem::Nested {
            id,
            uuid,
            label: StringRef::new(label),
            children: children.into(),
        }))
    } else if type_ == "separator" {
        Ok(ItemOrSeparator::Separator)
    } else if !enabled {
        Ok(ItemOrSeparator::Item(TrayItem::Disabled {
            id,
            uuid,
            label: StringRef::new(label),
        }))
    } else if toggle_type == "checkmark" {
        Ok(ItemOrSeparator::Item(TrayItem::Checkbox {
            id,
            uuid,
            label: StringRef::new(label),
            checked: toggle_state == 1,
        }))
    } else if toggle_type == "radio" {
        Ok(ItemOrSeparator::Item(TrayItem::Radio {
            id,
            uuid,
            label: StringRef::new(label),
            selected: toggle_state == 1,
        }))
    } else {
        Ok(ItemOrSeparator::Item(TrayItem::Regular {
            id,
            uuid,
            label: StringRef::new(label),
        }))
    }
}

#[derive(Debug)]
enum ItemOrSeparator {
    Item(TrayItem),
    Separator,
    Skip,
}

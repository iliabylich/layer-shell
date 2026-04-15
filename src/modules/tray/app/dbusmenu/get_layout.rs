use crate::{
    modules::{TrayItem, tray::uuid::UUID},
    utils::StringRef,
};
use anyhow::{Context, Result};
use mini_sansio_dbus::{
    IncomingArrayValue, IncomingValue, MethodCall, OutgoingCompleteType, OutgoingMessage,
    OutgoingValue, value_is,
};

pub(crate) const GET_LAYOUT: MethodCall<(StringRef, StringRef), Vec<TrayItem>, StringRef> =
    MethodCall::builder()
        .send(&|(destination, path): (StringRef, StringRef), _service| {
            OutgoingMessage::MethodCall {
                destination: Some(destination.to_string()),
                path: path.to_string(),
                interface: Some(String::from("com.canonical.dbusmenu")),
                serial: 0,
                member: String::from("GetLayout"),
                sender: None,
                unix_fds: None,
                body: vec![
                    OutgoingValue::Int32(0),
                    OutgoingValue::Int32(1),
                    OutgoingValue::Array(
                        OutgoingCompleteType::String,
                        vec![
                            OutgoingValue::String(String::from("type")),
                            OutgoingValue::String(String::from("label")),
                            OutgoingValue::String(String::from("enabled")),
                            OutgoingValue::String(String::from("visible")),
                            OutgoingValue::String(String::from("icon-name")),
                            OutgoingValue::String(String::from("icon-data")),
                            OutgoingValue::String(String::from("shortcut")),
                            OutgoingValue::String(String::from("toggle-type")),
                            OutgoingValue::String(String::from("toggle-state")),
                            OutgoingValue::String(String::from("children-display")),
                        ],
                    ),
                ],
            }
        })
        .try_process(&|mut body, service| {
            let _ = body.try_next()?.context("no root item id")?;
            let root = body.try_next()?.context("no root")?;
            value_is!(root, IncomingValue::Struct(root));

            let mut iter = root.iter()?;
            let _ = iter.try_next()?.context("expected 3 items")?;
            let _ = iter.try_next()?.context("expected 3 items")?;
            let top_level_items = iter.try_next()?.context("expected 3 items")?;

            value_is!(top_level_items, IncomingValue::Array(top_level_items));

            parse_items(service, top_level_items).map_err(|err| err.into())
        });

fn parse_items(service: StringRef, items: IncomingArrayValue<'_>) -> Result<Vec<TrayItem>> {
    let mut out = vec![];
    let mut batch = vec![];
    let mut iter = items.iter();

    while let Some(item) = iter.try_next()? {
        value_is!(item, IncomingValue::Variant(item));
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

fn parse_item(service: StringRef, item: IncomingValue<'_>) -> Result<ItemOrSeparator> {
    value_is!(item, IncomingValue::Struct(fields));

    let mut fields_iter = fields.iter()?;

    let id = fields_iter.try_next()?.context("expected 3 items")?;
    value_is!(id, IncomingValue::Int32(id));
    let uuid = UUID::encode(service.clone(), id);

    let props = fields_iter.try_next()?.context("expected 3 items")?;
    value_is!(props, IncomingValue::Array(props));
    let mut props_iter = props.iter();
    let mut type_ = "standard";
    let mut label = "";
    let mut enabled = true;
    let mut visible = true;
    let mut toggle_type = "";
    let mut toggle_state = -1;
    let mut children_display = "";
    while let Some(prop) = props_iter.try_next()? {
        value_is!(prop, IncomingValue::DictEntry(dict_entry));
        let (key, value) = dict_entry.key_value()?;
        value_is!(key, IncomingValue::String(key));
        value_is!(value, IncomingValue::Variant(value));

        match key {
            "type" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::String(value));
                type_ = value;
            }
            "label" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::String(value));
                label = value;
            }
            "enabled" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::Bool(value));
                enabled = value;
            }
            "visible" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::Bool(value));
                visible = value;
            }
            "toggle-type" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::String(value));
                toggle_type = value;
            }
            "toggle-state" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::Int32(value));
                toggle_state = value;
            }
            "children-display" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::String(value));
                children_display = value;
            }

            _ => log::warn!(target: "Tray", "Unknown dbusmenu prop: {key}"),
        }
    }

    let children_values = fields_iter.try_next()?.context("expected 3 items")?;
    value_is!(children_values, IncomingValue::Array(children_values));
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

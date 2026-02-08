use crate::{
    dbus::{
        Message, OneshotResource,
        messages::{type_is, value_is},
        types::{CompleteType, Value},
    },
    modules::{TrayItem, tray::uuid::UUID},
};
use anyhow::{Context, Result};
use std::borrow::Cow;

pub(crate) struct GetLayout {
    service: String,
    menu: String,
}

impl GetLayout {
    pub(crate) fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            menu: String::new(),
        }
    }
}

impl OneshotResource for GetLayout {
    type Input = (String, String);
    type Output = Vec<TrayItem>;

    fn make_request(&self, (destination, path): (String, String)) -> Message<'static> {
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
            destination: Some(Cow::Owned(destination)),
            path: Cow::Owned(path),
            interface: Some(Cow::Borrowed("com.canonical.dbusmenu")),
            serial: 0,
            member: Cow::Borrowed("GetLayout"),
            sender: None,
            unix_fds: None,
            body,
        }
    }

    fn try_process(&self, body: &[Value]) -> Result<Self::Output> {
        // log::error!("starting parsing...");

        let root: &[Value; 2] = body
            .try_into()
            .with_context(|| format!("expected 2 elements, got {}", body.len()))?;
        let root = &root[1];
        value_is!(root, Value::Struct(fields));
        let fields: &[Value; 3] = fields
            .as_slice()
            .try_into()
            .with_context(|| format!("expected 3 fields, got {}", fields.len()))?;
        value_is!(
            &fields[2],
            Value::Array(CompleteType::Variant, top_level_items)
        );

        // log::error!("starting parsing (2)...");
        parse_items(&self.service, &self.menu, top_level_items)
    }
}

pub(crate) fn parse_items(service: &str, menu: &str, items: &[Value]) -> Result<Vec<TrayItem>> {
    let mut out = vec![];
    let mut batch = vec![];

    for item in items {
        value_is!(item, Value::Variant(item));
        let item = parse_item(service, menu, item)?;
        // log::error!("item = {item:?}");
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

fn parse_item(service: &str, menu: &str, item: &Value) -> Result<ItemOrSeparator> {
    // log::error!("1 {item:?}");
    value_is!(item, Value::Struct(fields));
    let fields: &[Value; 3] = fields
        .as_slice()
        .try_into()
        .with_context(|| format!("expected 3 fields, got {}", fields.len()))?;

    // log::error!("2");
    value_is!(&fields[0], Value::Int32(id));
    let id = *id;
    let uuid = UUID::encode(service, menu, id);
    // log::error!("=== {id}");

    // log::error!("3");
    value_is!(
        &fields[1],
        Value::Array(CompleteType::DictEntry(key_t, value_t), props)
    );
    type_is!(&**key_t, CompleteType::String);
    type_is!(&**value_t, CompleteType::Variant);
    let mut type_ = &Cow::Borrowed("standard");
    let mut label = &Cow::Borrowed("");
    let mut enabled = true;
    let mut visible = true;
    let mut toggle_type = &Cow::Borrowed("");
    let mut toggle_state = -1;
    let mut children_display = &Cow::Borrowed("");
    for prop in props {
        value_is!(prop, Value::DictEntry(key, value));
        value_is!(&**key, Value::String(key));
        value_is!(&**value, Value::Variant(value));
        // log::error!("   {key} = {value:?}");

        match key.as_ref() {
            "type" => {
                value_is!(&**value, Value::String(value));
                type_ = value;
            }
            "label" => {
                value_is!(&**value, Value::String(value));
                label = value;
            }
            "enabled" => {
                value_is!(&**value, Value::Bool(value));
                enabled = *value;
            }
            "visible" => {
                value_is!(&**value, Value::Bool(value));
                visible = *value;
            }
            "toggle-type" => {
                value_is!(&**value, Value::String(value));
                toggle_type = value;
            }
            "toggle-state" => {
                value_is!(&**value, Value::Int32(value));
                toggle_state = *value;
            }
            "children-display" => {
                value_is!(&**value, Value::String(value));
                children_display = value;
            }

            _ => log::error!("Unknown dbusmenu prop: {key}"),
        }
    }

    // log::error!("4");
    value_is!(
        &fields[2],
        Value::Array(CompleteType::Variant, children_values)
    );
    let children = parse_items(service, menu, children_values)?;

    if !visible {
        Ok(ItemOrSeparator::Skip)
    } else if children_display == "submenu" {
        Ok(ItemOrSeparator::Item(TrayItem::Nested {
            id,
            uuid: uuid.into(),
            label: label.to_string().into(),
            children: children.into(),
        }))
    } else if type_ == "separator" {
        Ok(ItemOrSeparator::Separator)
    } else if !enabled {
        Ok(ItemOrSeparator::Item(TrayItem::Disabled {
            id,
            uuid: uuid.into(),
            label: label.to_string().into(),
        }))
    } else if toggle_type == "checkmark" {
        Ok(ItemOrSeparator::Item(TrayItem::Checkbox {
            id,
            uuid: uuid.into(),
            label: label.to_string().into(),
            checked: toggle_state == 1,
        }))
    } else if toggle_type == "radio" {
        Ok(ItemOrSeparator::Item(TrayItem::Radio {
            id,
            uuid: uuid.into(),
            label: label.to_string().into(),
            selected: toggle_state == 1,
        }))
    } else {
        Ok(ItemOrSeparator::Item(TrayItem::Regular {
            id,
            uuid: uuid.into(),
            label: label.to_string().into(),
        }))
    }
}

#[derive(Debug)]
enum ItemOrSeparator {
    Item(TrayItem),
    Separator,
    Skip,
}

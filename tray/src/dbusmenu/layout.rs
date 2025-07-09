use crate::{TrayItem, dbusmenu::proxy::DBusMenuProxy, uuid::Uuid};
use anyhow::{Context as _, Result};
use std::collections::HashMap;
use zbus::{
    Connection,
    zvariant::{Dict, OwnedObjectPath, OwnedValue, Str, Structure},
};

pub(crate) struct Layout<'a> {
    service: &'a str,
    menu: &'a str,
}

type Input = (u32, (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>));

impl<'a> Layout<'a> {
    pub(crate) async fn get(
        conn: Connection,
        service: &'a str,
        menu: &'a OwnedObjectPath,
    ) -> Result<Vec<TrayItem>> {
        let dbus_menu_proxy = DBusMenuProxy::builder(&conn)
            .destination(service.to_string())?
            .path(menu.clone())?
            .build()
            .await?;

        let input = dbus_menu_proxy
            .get_layout(
                0,
                10,
                &[
                    "type",
                    "label",
                    "enabled",
                    "visible",
                    "icon-name",
                    "icon-data",
                    "shortcut",
                    "toggle-type",
                    "toggle-state",
                    "children-display",
                ],
            )
            .await?;

        Self::parse(service, menu.as_str(), input)
    }

    fn parse(service: &'a str, menu: &'a str, input: Input) -> Result<Vec<TrayItem>> {
        Self { service, menu }.parse_input(input)
    }

    fn parse_input(&self, input: Input) -> Result<Vec<TrayItem>> {
        let (_, (_, _, items)) = input;
        self.parse_items(items)
    }

    fn parse_items(&self, items: Vec<OwnedValue>) -> Result<Vec<TrayItem>> {
        let mut out = vec![];
        let mut batch = vec![];

        for item in items {
            let item = self.parse_item(item)?;
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

    fn parse_item(&self, item: OwnedValue) -> Result<ItemOrSeparator> {
        let structure = Structure::try_from(item).context("expected structure")?;
        let mut fields = structure.into_fields().into_iter();

        let field1 = fields.next().context("expected exactly 3 children")?;
        let field2 = fields.next().context("expected exactly 3 children")?;
        let field3 = fields.next().context("expected exactly 3 children")?;

        let id = i32::try_from(field1).context("fields[0] must be i32")?;
        let uuid = Uuid::encode(self.service, self.menu, id);
        let props = Dict::try_from(field2).context("fields[1] must be a Dict")?;
        let children =
            Vec::<OwnedValue>::try_from(field3).context("fields[2] must be an array of values")?;

        let props = Props::try_from(props)?;

        if !props.visible {
            return Ok(ItemOrSeparator::Skip);
        }

        if props.children_display == "submenu" {
            let children = self.parse_items(children)?;
            Ok(ItemOrSeparator::Item(TrayItem::Nested {
                id,
                uuid: uuid.into(),
                label: props.label.into(),
                children: children.into(),
            }))
        } else if props.type_ == "separator" {
            Ok(ItemOrSeparator::Separator)
        } else if !props.enabled {
            Ok(ItemOrSeparator::Item(TrayItem::Disabled {
                id,
                uuid: uuid.into(),
                label: props.label.into(),
            }))
        } else if props.toggle_type == "checkmark" {
            Ok(ItemOrSeparator::Item(TrayItem::Checkbox {
                id,
                uuid: uuid.into(),
                label: props.label.into(),
                checked: props.toggle_state == 1,
            }))
        } else if props.toggle_type == "radio" {
            Ok(ItemOrSeparator::Item(TrayItem::Radio {
                id,
                uuid: uuid.into(),
                label: props.label.into(),
                selected: props.toggle_state == 1,
            }))
        } else {
            Ok(ItemOrSeparator::Item(TrayItem::Regular {
                id,
                uuid: uuid.into(),
                label: props.label.into(),
            }))
        }
    }
}

enum ItemOrSeparator {
    Item(TrayItem),
    Separator,
    Skip,
}

const PROP_TYPE: &Str = &Str::from_static("type");
const PROP_LABEL: &Str = &Str::from_static("label");
const PROP_ENABLED: &Str = &Str::from_static("enabled");
const PROP_VISIBLE: &Str = &Str::from_static("visible");
const PROP_TOGGLE_TYPE: &Str = &Str::from_static("toggle-type");
const PROP_TOGGLE_STATE: &Str = &Str::from_static("toggle-state");
const PROP_CHILDREN_DISPLAY: &Str = &Str::from_static("children-display");

#[derive(Debug)]
struct Props {
    type_: String,
    label: String,
    enabled: bool,
    visible: bool,
    toggle_type: String,
    toggle_state: i32,
    children_display: String,
}

impl TryFrom<Dict<'_, '_>> for Props {
    type Error = anyhow::Error;

    fn try_from(dict: Dict<'_, '_>) -> Result<Self> {
        let str_prop = |prop: &Str, fallback: &str| -> Result<String> {
            let value = dict
                .get::<_, Str>(prop)
                .with_context(|| format!("failed to get str key {prop}"))?;
            Ok(value
                .map(|s| s.to_string())
                .unwrap_or_else(|| fallback.to_string()))
        };

        let bool_prop = |prop: &Str, fallback: bool| -> Result<bool> {
            let value = dict
                .get::<_, bool>(prop)
                .with_context(|| format!("failed to get bool key {prop}"))?;
            Ok(value.unwrap_or(fallback))
        };

        let i32_prop = |prop: &Str, fallback: i32| -> Result<i32> {
            let value = dict
                .get::<_, i32>(prop)
                .with_context(|| format!("failed to get int key {prop}"))?;
            Ok(value.unwrap_or(fallback))
        };

        let type_ = str_prop(PROP_TYPE, "standard")?;
        let label = str_prop(PROP_LABEL, "")?;
        let enabled = bool_prop(PROP_ENABLED, true)?;
        let visible = bool_prop(PROP_VISIBLE, true)?;
        let toggle_type = str_prop(PROP_TOGGLE_TYPE, "")?;
        let toggle_state = i32_prop(PROP_TOGGLE_STATE, -1)?;
        let children_display = str_prop(PROP_CHILDREN_DISPLAY, "")?;

        Ok(Props {
            type_,
            label,
            enabled,
            visible,
            toggle_type,
            toggle_state,
            children_display,
        })
    }
}

use crate::{TrayItem, dbusmenu::proxy::DBusMenuProxy, uuid::Uuid};
use anyhow::{Context as _, Result};
use ffi::CString;
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
    ) -> Result<TrayItem> {
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

    fn parse(service: &'a str, menu: &'a str, input: Input) -> Result<TrayItem> {
        Self { service, menu }.parse_input(input)
    }

    fn parse_input(&self, input: Input) -> Result<TrayItem> {
        let (_, (_, _, items)) = input;
        let items = self.parse_items(items)?;

        Ok(TrayItem {
            children: items.into(),
            ..Default::default()
        })
    }

    fn parse_items(&self, items: Vec<OwnedValue>) -> Result<Vec<TrayItem>> {
        items
            .into_iter()
            .map(|item| self.parse_item(item))
            .collect::<Result<Vec<_>>>()
    }

    fn parse_item(&self, item: OwnedValue) -> Result<TrayItem> {
        let structure = Structure::try_from(item).context("expected structure")?;
        let mut fields = structure.into_fields().into_iter();

        let field1 = fields.next().context("expected exactly 3 children")?;
        let field2 = fields.next().context("expected exactly 3 children")?;
        let field3 = fields.next().context("expected exactly 3 children")?;

        let id = i32::try_from(field1).context("fields[0] must be i32")?;
        let props = Dict::try_from(field2).context("fields[1] must be a Dict")?;
        let children =
            Vec::<OwnedValue>::try_from(field3).context("fields[2] must be an array of values")?;

        let props = Props::try_from(props)?;
        let children = self.parse_items(children)?;

        Ok(TrayItem {
            id,
            uuid: Uuid::encode(self.service, self.menu, id).into(),
            type_: props.type_,
            label: props.label,
            enabled: props.enabled,
            visible: props.visible,
            icon_name: props.icon_name,
            icon_data: props.icon_data,
            toggle_type: props.toggle_type,
            toggle_state: props.toggle_state,
            children_display: props.children_display,
            children: children.into(),
        })
    }
}

const PROP_TYPE: &Str = &Str::from_static("type");
const PROP_LABEL: &Str = &Str::from_static("label");
const PROP_ENABLED: &Str = &Str::from_static("enabled");
const PROP_VISIBLE: &Str = &Str::from_static("visible");
const PROP_ICON_NAME: &Str = &Str::from_static("icon-name");
const PROP_ICON_DATA: &Str = &Str::from_static("icon-data");
const PROP_TOGGLE_TYPE: &Str = &Str::from_static("toggle-type");
const PROP_TOGGLE_STATE: &Str = &Str::from_static("toggle-state");
const PROP_CHILDREN_DISPLAY: &Str = &Str::from_static("children-display");

struct Props {
    type_: CString,
    label: CString,
    enabled: bool,
    visible: bool,
    icon_name: CString,
    icon_data: CString,
    toggle_type: CString,
    toggle_state: i32,
    children_display: CString,
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
        let icon_name = str_prop(PROP_ICON_NAME, "")?;
        let icon_data = str_prop(PROP_ICON_DATA, "")?;
        let toggle_type = str_prop(PROP_TOGGLE_TYPE, "")?;
        let toggle_state = i32_prop(PROP_TOGGLE_STATE, -1)?;
        let children_display = str_prop(PROP_CHILDREN_DISPLAY, "")?;

        Ok(Props {
            type_: type_.into(),
            label: label.into(),
            enabled,
            visible,
            icon_name: icon_name.into(),
            icon_data: icon_data.into(),
            toggle_type: toggle_type.into(),
            toggle_state,
            children_display: children_display.into(),
        })
    }
}

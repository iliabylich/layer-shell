use crate::{
    dbus::{gen::dbus_menu::ComCanonicalDbusmenu, tray::UUID},
    event::TrayItem,
};
use anyhow::{bail, Context as _, Result};
use dbus::{
    arg::{RefArg, Variant},
    blocking::{Connection, Proxy},
};
use std::time::Duration;

pub(crate) struct DBusMenu {
    service: String,
    path: String,
}

impl DBusMenu {
    pub(crate) fn new(service: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            path: path.into(),
        }
    }

    fn proxy<'a>(&'a self, conn: &'a Connection) -> Proxy<'a, &'a Connection> {
        Proxy::new(&self.service, &self.path, Duration::from_millis(5000), conn)
    }

    pub(crate) fn get_layout(&self, conn: &Connection) -> Result<TrayItem> {
        let (_, (_, _, items)) = self
            .proxy(conn)
            .get_layout(
                0,
                10,
                vec![
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
            .context("Failed to call GetLayout")?;

        let children = self.visit_list(items)?;

        Ok(TrayItem {
            id: 0,
            uuid: String::new().into(),
            type_: String::new().into(),
            label: String::new().into(),
            enabled: true,
            visible: true,
            icon_name: String::new().into(),
            icon_data: String::new().into(),
            toggle_type: String::new().into(),
            toggle_state: 0,
            children_display: String::new().into(),
            children: children.into(),
        })
    }

    pub(crate) fn event(&self, conn: &Connection, id: i32) -> Result<()> {
        self.proxy(conn)
            .event(
                id,
                "clicked",
                Variant(Box::new(0i32) as Box<dyn RefArg>),
                10118055,
            )
            .context("failed to call Event")
    }

    fn visit_list(&self, items: Vec<V>) -> Result<Vec<TrayItem>> {
        items
            .into_iter()
            .map(|v| self.visit_node(v.0))
            .collect::<Result<Vec<_>, _>>()
    }

    fn visit_node(&self, node: Box<dyn RefArg>) -> Result<TrayItem> {
        let mut id = None;
        let mut type_ = "standard".to_string();
        let mut label = "".to_string();
        let mut enabled = true;
        let mut visible = true;
        let mut icon_name = "".to_string();
        let mut icon_data = "".to_string();
        let mut toggle_type = "".to_string();
        let mut toggle_state = -1;
        let mut children_display = "".to_string();
        let mut children = vec![];

        for (idx, item) in node
            .as_iter()
            .context("inner node is not an iter")?
            .enumerate()
        {
            let item = item.box_clone();
            if idx == 0 {
                id = Some(*dbus::arg::cast::<i32>(&item).context("id is not an int")?);
            } else if idx == 1 {
                let mut iter = item.as_iter().context("properties are not an iter")?;

                while let Some(key) = iter.next() {
                    let value = iter
                        .next()
                        .context("expected even number of elements in the properties hash")?;

                    let key = key.as_str().context("key is not a string")?.to_string();

                    match &key[..] {
                        "type" => type_ = visit_str_property("type", value)?,
                        "label" => label = visit_str_property("label", value)?,
                        "enabled" => enabled = visit_bool_property("enabled", value)?,
                        "visible" => visible = visit_bool_property("visible", value)?,
                        "icon-name" => icon_name = visit_str_property("icon-name", value)?,
                        "icon-data" => icon_data = visit_str_property("icon-data", value)?,
                        "toggle-type" => toggle_type = visit_str_property("toggle-type", value)?,
                        "toggle-state" => toggle_state = visit_int_property("toggle-state", value)?,
                        "children-display" => {
                            children_display = visit_str_property("children-display", value)?
                        }
                        "shortcut" => {
                            // ignore
                        }
                        _ => bail!("unsupported property {key}"),
                    }
                }
            } else if idx == 2 {
                for child in item.as_iter().context("not an iter(4)")? {
                    for child in child.as_iter().context("child is not an iter")? {
                        let child = child.box_clone();
                        let child = self.visit_node(child).context("invalid child")?;
                        children.push(child);
                    }
                }
            } else {
                bail!("expected 3 elements in each layout node")
            }
        }

        let id = id.context("no id found")?;

        Ok(TrayItem {
            id,
            uuid: UUID::encode(&self.service, &self.path, id).into(),
            type_: type_.into(),
            label: label.into(),
            enabled,
            visible,
            icon_name: icon_name.into(),
            icon_data: icon_data.into(),
            toggle_type: toggle_type.into(),
            toggle_state,
            children_display: children_display.into(),
            children: children.into(),
        })
    }
}

fn visit_str_property(key: &str, value: &dyn RefArg) -> Result<String> {
    Ok(value
        .as_str()
        .with_context(|| format!("{key} property is not a string"))?
        .to_string())
}

fn visit_bool_property(key: &str, value: &dyn RefArg) -> Result<bool> {
    let n = value
        .as_i64()
        .with_context(|| format!("{key} property is not a bool"))?;
    Ok(n != 0)
}

fn visit_int_property(key: &str, value: &dyn RefArg) -> Result<i64> {
    value
        .as_i64()
        .with_context(|| format!("{key} property is not a bool"))
}

type V = Variant<Box<dyn RefArg + 'static>>;

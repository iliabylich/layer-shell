use crate::dbus::{gen::dbus_menu::ComCanonicalDbusmenu, tray::UUID};
use anyhow::{bail, ensure, Context as _, Result};
use dbus::{
    arg::{cast, RefArg, Variant},
    blocking::{Connection, Proxy},
};
use std::time::Duration;

pub(crate) struct DBusMenu {
    service: String,
    path: String,
}

pub(crate) struct DBusMenuItem {
    pub(crate) label: String,
    pub(crate) disabled: bool,
    pub(crate) uuid: String,
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

    pub(crate) fn get_layout(&self, conn: &Connection) -> Result<Vec<DBusMenuItem>> {
        let (_, raw_tree) = self
            .proxy(conn)
            .get_layout(0, 10, vec!["label", "enabled", "visible"])
            .context("Failed to call GetLayout")?;

        let mut items = vec![];

        visit_root(raw_tree, &mut items, &self.service, &self.path)?;

        Ok(items)
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
}

type DBusInternalTree = (
    i32,
    dbus::arg::PropMap,
    Vec<dbus::arg::Variant<Box<dyn dbus::arg::RefArg + 'static>>>,
);

fn visit_root(
    root: DBusInternalTree,
    out: &mut Vec<DBusMenuItem>,
    service: &str,
    path: &str,
) -> Result<()> {
    let (_root_id, _props, children) = root;

    visit_children(children, out, service, path)?;
    Ok(())
}

fn visit_children(
    children: Vec<Variant<Box<dyn RefArg>>>,
    out: &mut Vec<DBusMenuItem>,
    service: &str,
    path: &str,
) -> Result<()> {
    for child in children {
        visit_child(child, out, service, path)?;
    }
    Ok(())
}

fn visit_child(
    child: Variant<Box<dyn RefArg>>,
    out: &mut Vec<DBusMenuItem>,
    service: &str,
    path: &str,
) -> Result<()> {
    if let Some(mut iter) = child.as_iter() {
        if let Some(child) = iter.next() {
            if let Some(mut iter) = child.as_iter() {
                let triplet = iter.next().zip(iter.next()).zip(iter.next());
                if let Some(((v1, v2), v3)) = triplet {
                    visit_triplet(
                        v1.box_clone(),
                        v2.box_clone(),
                        v3.box_clone(),
                        out,
                        service,
                        path,
                    )?;
                }
            }
        }
    }

    Ok(())
}

fn visit_triplet(
    v1: Box<dyn RefArg>,
    v2: Box<dyn RefArg>,
    v3: Box<dyn RefArg>,
    out: &mut Vec<DBusMenuItem>,
    service: &str,
    path: &str,
) -> Result<()> {
    if let Some(id) = cast::<i32>(&v1).copied() {
        if let Some(props) = Props::parse(v2)? {
            out.push(DBusMenuItem {
                label: props.label,
                disabled: !props.enabled,
                uuid: UUID::encode(service, path, id),
            });

            if let Some(mut iter) = v3.as_iter() {
                ensure!(iter.next().is_none());
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Props {
    label: String,
    enabled: bool,
}

impl Props {
    fn parse(props: Box<dyn RefArg>) -> Result<Option<Self>> {
        let mut label = None;
        let mut enabled = true;
        let mut visible = true;

        let mut iter = props.as_iter().context("Props are not iterable")?;

        while let Some(key) = iter.next() {
            let value = iter
                .next()
                .context("Odd number of elements in the DBus Hash")?;

            if let Some(key) = key.as_str() {
                match key {
                    "label" => {
                        label = Some(
                            value
                                .as_str()
                                .context("DBus menu item has no name, skipping")?
                                .to_string(),
                        )
                    }
                    "visible" => {
                        visible = value.as_i64().map(|v| v != 0).unwrap_or(true);
                    }
                    "enabled" => {
                        enabled = value.as_i64().map(|v| v != 0).unwrap_or(true);
                    }
                    _ => {
                        bail!("Unsupported key: {key}")
                    }
                }
            }
        }

        if !visible {
            return Ok(None);
        }

        if let Some(label) = label {
            Ok(Some(Self { label, enabled }))
        } else {
            Ok(None)
        }
    }
}

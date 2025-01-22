use crate::{
    dbus::gen::dbus_menu::ComCanonicalDbusmenu,
    event::{TrayApp, TrayIcon, TrayItem},
};
use anyhow::{Context as _, Result};
use dbus::{
    arg::{cast, RefArg, Variant},
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

    pub(crate) fn get_layout(&self, conn: &Connection) -> Result<TrayApp> {
        let (_, raw_tree) = self
            .proxy(conn)
            .get_layout(0, 10, vec!["label", "enabled", "visible"])
            .context("Failed to call GetLayout")?;

        let mut items = vec![];

        visit_root(raw_tree, &mut items);

        Ok(TrayApp {
            items: items.into(),
            icon: TrayIcon::None,
        })
    }
}

type DBusInternalTree = (
    i32,
    dbus::arg::PropMap,
    Vec<dbus::arg::Variant<Box<dyn dbus::arg::RefArg + 'static>>>,
);

fn visit_root(root: DBusInternalTree, out: &mut Vec<TrayItem>) {
    let (_root_id, _props, children) = root;

    visit_children(children, out);
}

fn visit_children(children: Vec<Variant<Box<dyn RefArg>>>, out: &mut Vec<TrayItem>) {
    for child in children {
        visit_child(child, out);
    }
}

fn visit_child(child: Variant<Box<dyn RefArg>>, out: &mut Vec<TrayItem>) {
    if let Some(mut iter) = child.as_iter() {
        if let Some(child) = iter.next() {
            if let Some(mut iter) = child.as_iter() {
                let triplet = iter.next().zip(iter.next()).zip(iter.next());
                if let Some(((v1, v2), v3)) = triplet {
                    visit_triplet(v1.box_clone(), v2.box_clone(), v3.box_clone(), out);
                }
            }
        }
    }
}

fn visit_triplet(
    v1: Box<dyn RefArg>,
    v2: Box<dyn RefArg>,
    v3: Box<dyn RefArg>,
    out: &mut Vec<TrayItem>,
) {
    if let Some(id) = cast(&v1).copied() {
        if let Some(props) = Props::parse(v2) {
            out.push(TrayItem {
                label: props.label.into(),
                id,
            });

            if let Some(iter) = v3.as_iter() {
                for child in iter {
                    log::error!("Nested child: {:?}, unsupported as of now", child);
                }
            }
        }
    }
}

#[derive(Debug)]
struct Props {
    label: String,
    enabled: bool,
    visible: bool,
}

impl Props {
    fn parse(props: Box<dyn RefArg>) -> Option<Self> {
        let mut out = Self {
            label: String::from("<none>"),
            enabled: true,
            visible: true,
        };

        let mut iter = match props.as_iter() {
            Some(iter) => iter,
            None => {
                log::error!("Props are not an interable");
                return None;
            }
        };

        while let Some(key) = iter.next() {
            let Some(value) = iter.next() else {
                log::error!("Odd number of elements in the DBus Hash");
                return None;
            };

            if let Some(key) = key.as_str() {
                match key {
                    "label" => match value.as_str() {
                        Some(label) => out.label = label.to_string(),
                        None => {
                            log::error!("DBus menu item has no name, skipping");
                            return None;
                        }
                    },
                    "visible" => {
                        out.visible = value.as_i64().map(|v| v != 0).unwrap_or(true);
                    }
                    "enabled" => {
                        out.enabled = value.as_i64().map(|v| v != 0).unwrap_or(true);
                    }
                    _ => {
                        log::error!("Unsupported key: {key}");
                        return None;
                    }
                }
            }
        }

        Some(out)
    }
}

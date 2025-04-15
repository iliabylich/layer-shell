use crate::{dbus::generated::status_notifier_item::OrgKdeStatusNotifierItem, event::TrayIcon};
use anyhow::{Context as _, Result};
use dbus::blocking::{Connection, Proxy};
use std::time::Duration;

#[derive(Debug)]
pub(crate) struct StatusNotifierItem {
    service: String,
    path: String,
}

impl StatusNotifierItem {
    pub(crate) fn new(service: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            path: path.into(),
        }
    }

    fn proxy<'a>(&'a self, conn: &'a Connection) -> Proxy<'a, &'a Connection> {
        Proxy::new(&self.service, &self.path, Duration::from_millis(5000), conn)
    }

    pub(crate) fn icon_name(&self, conn: &Connection) -> Result<TrayIcon> {
        let name_or_path = self
            .proxy(conn)
            .icon_name()
            .context("failed to get IconName")?;

        if name_or_path.starts_with("/") {
            Ok(TrayIcon::Path { path: name_or_path })
        } else {
            Ok(TrayIcon::Name { name: name_or_path })
        }
    }

    pub(crate) fn icon_pixmap(&self, conn: &Connection) -> Result<TrayIcon> {
        let variants = self
            .proxy(conn)
            .icon_pixmap()
            .context("failed to get IconPixmap")?;

        let (w, h, bytes) = variants
            .into_iter()
            .max_by(|(w1, _, _), (w2, _, _)| w1.cmp(w2))
            .context("DBus returned IconPixmap but it has no variants")?;

        Ok(TrayIcon::PixmapVariant {
            w: w as u32,
            h: h as u32,
            bytes,
        })
    }

    pub(crate) fn any_icon(&self, conn: &Connection) -> TrayIcon {
        self.icon_name(conn)
            .or_else(|_| self.icon_pixmap(conn))
            .unwrap_or_else(|_| {
                log::warn!("DBus service has no IconName/IconPixmap");
                TrayIcon::Unset()
            })
    }

    pub(crate) fn menu(&self, conn: &Connection) -> Result<String> {
        let menu = self.proxy(conn).menu().context("failed to get Menu")?;
        Ok(menu.to_string())
    }
}

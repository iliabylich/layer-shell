use crate::{
    TrayCtl, TrayEvent, TrayIcon, TrayIconPixmap, TrayItem,
    dbus::{NameLost, NameOwnerChanged},
    dbus_event::DBusEvent,
    dbusmenu::{DBusMenu, ItemsPropertiesUpdated, Layout, LayoutUpdated},
    multiplexer::Multiplexer,
    status_notifier_item::{IconName, IconPixmap, Menu, NewIcon},
    status_notifier_watcher::StatusNotifierWatcher,
    store::Store,
    tray_stream::TrayStream,
    uuid::Uuid,
};
use anyhow::Result;
use futures::{StreamExt, TryFutureExt as _};
use module::Module;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use zbus::{Connection, zvariant::OwnedObjectPath};

pub struct Tray {
    multiplexer: Multiplexer,
    token: CancellationToken,
    store: Store,
    etx: UnboundedSender<TrayEvent>,
    crx: UnboundedReceiver<String>,
}

#[async_trait::async_trait]
impl Module for Tray {
    const NAME: &str = "Tray";

    type Event = TrayEvent;
    type Command = String;
    type Ctl = TrayCtl;

    fn new(
        etx: UnboundedSender<Self::Event>,
        crx: UnboundedReceiver<Self::Command>,
        token: CancellationToken,
    ) -> Self {
        Self {
            multiplexer: Multiplexer::new(),
            token,
            store: Store::new(),
            etx,
            crx,
        }
    }

    async fn start(&mut self) -> Result<()> {
        let conn = Connection::session().await?;

        self.add::<StatusNotifierWatcher>(&conn, ()).await?;
        self.add::<NameLost>(&conn, ()).await?;
        self.add::<NameOwnerChanged>(&conn, ()).await?;

        loop {
            tokio::select! {
                Some((_stream_id, event)) = self.multiplexer.next() => {
                    if let Err(err) = self.on_event(&conn, event).await {
                        log::error!(target: "Tray", "{err:?}");
                    }
                }

                Some(uuid) = self.crx.recv() => {
                    if let Err(err) = self.trigger(&conn, uuid).await {
                        log::error!(target: "Tray", "{err:?}");
                    }
                }

                _ = self.token.cancelled() => {
                    log::info!(target: "Tray", "exiting...");
                    return Ok(())
                }
            }
        }
    }
}

impl Tray {
    async fn add<S: TrayStream>(&mut self, conn: &Connection, input: S::Input) -> Result<()> {
        let (id, stream) = S::stream(conn, input).await?;
        self.multiplexer.add(id, stream);
        Ok(())
    }

    async fn on_event(&mut self, conn: &Connection, event: DBusEvent) -> Result<()> {
        match event {
            DBusEvent::NameLost(name) => {
                self.on_name_lost(&name)?;
            }
            DBusEvent::NameOwnerChanged { name, new_owner } => {
                if new_owner.is_none() {
                    self.on_name_lost(&name)?;
                }
            }

            DBusEvent::ServiceAdded(service) => {
                self.on_service_added(conn, service).await?;
            }

            DBusEvent::IconNameChanged { service, icon_name } => {
                self.on_icon_name_changed(service, icon_name)?;
            }
            DBusEvent::IconPixmapChanged { service, pixmap } => {
                self.on_icon_pixmap_changed(service, pixmap).await?;
            }
            DBusEvent::MenuChanged { service, menu } => {
                self.on_menu_changed(conn, service, menu).await?;
            }
            DBusEvent::NewIconReceived { service } => {
                self.on_new_icon_received(conn, service).await?;
            }

            DBusEvent::LayoutUpdated { service, menu } => {
                self.on_layout_updated(conn, service, menu).await?;
            }
            DBusEvent::ItemsPropertiesUpdated { service, menu } => {
                self.on_items_properties_updated(service, menu)?;
            }

            DBusEvent::LayoutReceived { service, items } => {
                self.on_layout_received(service, items)?;
            }
        }
        Ok(())
    }

    async fn on_service_added(&mut self, conn: &Connection, service: Arc<str>) -> Result<()> {
        self.add::<IconName>(conn, Arc::clone(&service)).await?;
        self.add::<IconPixmap>(conn, Arc::clone(&service)).await?;
        self.add::<Menu>(conn, Arc::clone(&service)).await?;
        self.add::<NewIcon>(conn, Arc::clone(&service)).await?;

        Ok(())
    }

    fn on_name_lost(&mut self, service: &str) -> Result<()> {
        let Some(count_removed) = self.multiplexer.remove_service(service) else {
            return Ok(());
        };

        log::info!(target: "Tray", "{service} exited, removed {count_removed} streams");

        let service = Arc::from(service.to_string().into_boxed_str());
        if let Some(event) = self.store.remove(service) {
            self.etx.send(event)?;
        }
        Ok(())
    }

    fn on_icon_name_changed(&mut self, service: Arc<str>, icon_name: String) -> Result<()> {
        let icon = TrayIcon::detect_name_or_path(icon_name);
        if let Some(event) = self.store.update_icon(service, icon) {
            self.etx.send(event)?;
        }
        Ok(())
    }

    async fn on_icon_pixmap_changed(
        &mut self,
        service: Arc<str>,
        pixmap: TrayIconPixmap,
    ) -> Result<()> {
        let icon = TrayIcon::Pixmap(pixmap);
        if let Some(event) = self.store.update_icon(service, icon) {
            self.etx.send(event)?;
        }
        Ok(())
    }

    async fn on_menu_changed(
        &mut self,
        conn: &Connection,
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    ) -> Result<()> {
        self.add::<LayoutUpdated>(conn, (Arc::clone(&service), Arc::clone(&menu)))
            .await?;

        self.add::<ItemsPropertiesUpdated>(conn, (Arc::clone(&service), Arc::clone(&menu)))
            .await?;

        Ok(())
    }

    async fn on_new_icon_received(&mut self, conn: &Connection, service: Arc<str>) -> Result<()> {
        let fut1 = IconName::get(conn, &service).map_ok(|icon_name| DBusEvent::IconNameChanged {
            service: Arc::clone(&service),
            icon_name,
        });

        let fut2 = IconPixmap::get(conn, &service).map_ok(|pixmap| DBusEvent::IconPixmapChanged {
            service: Arc::clone(&service),
            pixmap,
        });

        let (e1, e2) = tokio::join!(fut1, fut2);

        let Ok(event) = e1.or(e2) else {
            log::error!(
                target: "Tray",
                "got notification about new icon but both IconName and IconPixmap are not set"
            );
            return Ok(());
        };
        self.multiplexer.emit(event)?;
        Ok(())
    }

    async fn on_layout_updated(
        &mut self,
        conn: &Connection,
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    ) -> Result<()> {
        let items = Layout::get(conn, &service, &menu).await?;
        self.multiplexer.emit(DBusEvent::LayoutReceived {
            items,
            service: Arc::clone(&service),
        })?;

        Ok(())
    }

    fn on_items_properties_updated(
        &mut self,
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    ) -> Result<()> {
        self.multiplexer
            .emit(DBusEvent::LayoutUpdated { service, menu })
    }

    fn on_layout_received(&mut self, service: Arc<str>, items: Vec<TrayItem>) -> Result<()> {
        if let Some(event) = self.store.update_item(service, items) {
            self.etx.send(event)?;
        }
        Ok(())
    }

    async fn trigger(&mut self, conn: &Connection, uuid: String) -> Result<()> {
        let (service, menu, id) = Uuid::decode(&uuid)?;
        DBusMenu::trigger(conn, service, menu, id).await?;
        Ok(())
    }
}

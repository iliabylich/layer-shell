use crate::{
    TrayEvent, TrayIcon, TrayItem,
    dbus::{NameLostEvent, NameOwnerChangedEvent},
    dbus_event::DBusEvent,
    dbusmenu::{ItemsPropertiesUpdated, Layout, LayoutUpdated, trigger_tray_item},
    multiplexer::Multiplexer,
    status_notifier_item::{IconName, IconPixmap, Menu, NewIcon},
    status_notifier_watcher::StatusNotifierWatcher,
    store::Store,
    tray_stream::TrayStream,
    uuid::Uuid,
};
use anyhow::Result;
use futures::{StreamExt, TryFutureExt};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use zbus::{Connection, zvariant::OwnedObjectPath};

pub(crate) struct TrayTask {
    multiplexer: Multiplexer,
    token: CancellationToken,
    conn: Connection,
    store: Store,
    etx: UnboundedSender<TrayEvent>,
    crx: UnboundedReceiver<String>,
}

impl TrayTask {
    pub(crate) async fn start(
        etx: UnboundedSender<TrayEvent>,
        crx: UnboundedReceiver<String>,
        token: CancellationToken,
    ) -> Result<()> {
        Self {
            multiplexer: Multiplexer::new(),
            token,
            conn: Connection::session().await?,
            store: Store::new(),
            etx,
            crx,
        }
        .r#loop()
        .await
    }

    async fn r#loop(mut self) -> Result<()> {
        self.add::<StatusNotifierWatcher>(()).await?;
        self.add::<NameLostEvent>(()).await?;
        self.add::<NameOwnerChangedEvent>(()).await?;

        loop {
            tokio::select! {
                Some((_stream_id, event)) = self.multiplexer.next() => {
                    if let Err(err) = self.on_event(event).await {
                        log::error!(target: "Tray", "{err:?}");
                    }
                }

                Some(uuid) = self.crx.recv() => {
                    if let Err(err) = self.trigger(uuid).await {
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

    async fn on_event(&mut self, event: DBusEvent) -> Result<()> {
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
                self.on_service_added(service).await?;
            }

            DBusEvent::IconNameChanged { service, icon_name } => {
                self.on_icon_name_changed(service, icon_name).await?;
            }
            DBusEvent::IconPixmapChanged {
                service,
                width,
                height,
                bytes,
            } => {
                self.on_icon_pixmap_changed(service, width, height, bytes)
                    .await?;
            }
            DBusEvent::MenuChanged { service, menu } => {
                self.on_menu_changed(service, menu).await?;
            }
            DBusEvent::NewIconReceived { service } => {
                self.on_new_icon_received(service).await?;
            }

            DBusEvent::LayoutUpdated { service, menu } => {
                self.on_layout_updated(service, menu).await?;
            }
            DBusEvent::ItemsPropertiesUpdated { service, menu } => {
                self.on_items_properties_updated(service, menu).await?;
            }

            DBusEvent::LayoutReceived { service, items } => {
                self.on_layout_received(service, items).await?;
            }
        }
        Ok(())
    }

    async fn on_service_added(&mut self, service: Arc<str>) -> Result<()> {
        self.add::<IconName>(Arc::clone(&service)).await?;
        self.add::<IconPixmap>(Arc::clone(&service)).await?;
        self.add::<Menu>(Arc::clone(&service)).await?;
        self.add::<NewIcon>(Arc::clone(&service)).await?;

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

    async fn on_icon_name_changed(&mut self, service: Arc<str>, icon_name: String) -> Result<()> {
        let icon = TrayIcon::detect_name_or_path(icon_name);
        if let Some(event) = self.store.update_icon(service, icon) {
            self.etx.send(event)?;
        }
        Ok(())
    }

    async fn on_icon_pixmap_changed(
        &mut self,
        service: Arc<str>,
        width: i32,
        height: i32,
        bytes: Vec<u8>,
    ) -> Result<()> {
        let icon = TrayIcon::new_pixmap(width, height, bytes);
        if let Some(event) = self.store.update_icon(service, icon) {
            self.etx.send(event)?;
        }
        Ok(())
    }

    async fn on_menu_changed(
        &mut self,
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    ) -> Result<()> {
        self.add::<LayoutUpdated>((Arc::clone(&service), Arc::clone(&menu)))
            .await?;

        self.add::<ItemsPropertiesUpdated>((Arc::clone(&service), Arc::clone(&menu)))
            .await?;

        Ok(())
    }

    async fn on_new_icon_received(&mut self, service: Arc<str>) -> Result<()> {
        let fut1 = IconName::get(&self.conn, Arc::clone(&service)).map_ok(|icon_name| {
            DBusEvent::IconNameChanged {
                service: Arc::clone(&service),
                icon_name,
            }
        });

        let fut2 =
            IconPixmap::get(&self.conn, Arc::clone(&service)).map_ok(|(width, height, bytes)| {
                DBusEvent::IconPixmapChanged {
                    service: Arc::clone(&service),
                    width,
                    height,
                    bytes,
                }
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
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    ) -> Result<()> {
        let items = Layout::get(self.conn.clone(), &service, &menu).await?;
        self.multiplexer.emit(DBusEvent::LayoutReceived {
            items,
            service: Arc::clone(&service),
        })?;

        Ok(())
    }

    async fn on_items_properties_updated(
        &mut self,
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    ) -> Result<()> {
        self.multiplexer
            .emit(DBusEvent::LayoutUpdated { service, menu })
    }

    async fn on_layout_received(&mut self, service: Arc<str>, items: Vec<TrayItem>) -> Result<()> {
        if let Some(event) = self.store.update_item(service, items) {
            self.etx.send(event)?;
        }
        Ok(())
    }

    async fn add<S: TrayStream>(&mut self, input: S::Input) -> Result<()> {
        let (id, stream) = S::stream(&self.conn, input).await?;
        self.multiplexer.add(id, stream);
        Ok(())
    }

    async fn trigger(&mut self, uuid: String) -> Result<()> {
        let (service, menu, id) = Uuid::decode(&uuid)?;
        trigger_tray_item(self.conn.clone(), service, menu, id).await?;
        Ok(())
    }
}

// TODO: ASYNC TRAIT SendEvent with impl for <Event = TrayEvent> and <Event = DBusEvent>

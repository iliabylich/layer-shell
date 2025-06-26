use crate::{
    TrayEvent, TrayIcon, TrayItem,
    dbus::{NameLostEvent, NameOwnerChangedEvent},
    dbus_event::DBusEvent,
    dbusmenu::{ItemsPropertiesUpdated, Layout, LayoutUpdated},
    status_notifier_item::{IconNameUpdated, IconPixmapUpdate, MenuUpdated},
    status_notifier_watcher::StatusNotifierWatcher,
    store::Store,
    stream_id::StreamId,
    stream_map::StreamMap,
};
use anyhow::Result;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;
use zbus::{Connection, zvariant::OwnedObjectPath};

pub(crate) struct TrayTask {
    stream_map: StreamMap,
    token: CancellationToken,
    conn: Connection,
    store: Store,
    tx: UnboundedSender<TrayEvent>,
}

impl TrayTask {
    pub(crate) async fn start(
        tx: UnboundedSender<TrayEvent>,
        token: CancellationToken,
    ) -> Result<()> {
        let mut stream_map = StreamMap::new();
        let conn = Connection::session().await?;

        stream_map.add(
            StreamId::ServiceAdded,
            StatusNotifierWatcher::into_stream(conn.clone()).await?,
        );

        stream_map.add(
            StreamId::NameLost,
            NameLostEvent::into_stream(conn.clone()).await?,
        );
        stream_map.add(
            StreamId::NameOwnedChanged,
            NameOwnerChangedEvent::into_stream(conn.clone()).await?,
        );

        Self {
            stream_map,
            token,
            conn,
            store: Store::new(),
            tx,
        }
        .r#loop()
        .await
    }

    async fn r#loop(mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some((_stream_id, event)) = self.stream_map.next() => {
                    self.on_event(event).await?;
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
                self.on_service_removed(&name)?;
            }
            DBusEvent::NameOwnerChanged { name, new_owner } => {
                if new_owner.is_none() {
                    self.on_service_removed(&name)?;
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

            DBusEvent::LayoutUpdated { service, menu } => {
                self.on_layout_updated(service, menu).await?;
            }
            DBusEvent::ItemsPropertiesUpdated { service, menu } => {
                self.on_items_properties_updated(service, menu).await?;
            }

            DBusEvent::LayoutReceived { service, item } => {
                self.on_layout_received(service, item).await?;
            }
        }
        Ok(())
    }

    async fn on_service_added(&mut self, service: Arc<str>) -> Result<()> {
        macro_rules! subscribe {
            ($stream:ident) => {
                let (event, stream_id, stream) =
                    $stream::split(self.conn.clone(), Arc::clone(&service)).await?;

                match event {
                    Ok(event) => self.stream_map.emit(event)?,
                    Err(err) => log::error!("{err:?}"),
                }

                self.stream_map.add(stream_id, stream);
            };
        }

        subscribe!(IconNameUpdated);
        subscribe!(IconPixmapUpdate);
        subscribe!(MenuUpdated);

        Ok(())
    }

    fn on_service_removed(&mut self, service: &str) -> Result<()> {
        let Some(count_removed) = self.stream_map.remove_service(&service) else {
            return Ok(());
        };

        log::info!(target: "Tray", "{service} exited, removed {count_removed} streams");

        let service = Arc::from(service.to_string().into_boxed_str());
        if let Some(event) = self.store.remove(service) {
            self.tx.send(event)?;
        }
        Ok(())
    }

    async fn on_icon_name_changed(&mut self, service: Arc<str>, icon_name: String) -> Result<()> {
        let icon = TrayIcon::from(icon_name);
        if let Some(event) = self.store.update_icon(service, icon) {
            self.tx.send(event)?;
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
        let icon = TrayIcon::from((width, height, bytes));
        if let Some(event) = self.store.update_icon(service, icon) {
            self.tx.send(event)?;
        }
        Ok(())
    }

    async fn on_menu_changed(
        &mut self,
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    ) -> Result<()> {
        macro_rules! subscribe {
            ($stream:ident) => {
                let (event, stream_id, stream) =
                    $stream::split(self.conn.clone(), Arc::clone(&service), Arc::clone(&menu))
                        .await?;

                if let Some(event) = event {
                    self.stream_map.emit(event)?;
                }

                self.stream_map.add(stream_id, stream);
            };
        }

        subscribe!(LayoutUpdated);
        subscribe!(ItemsPropertiesUpdated);

        Ok(())
    }

    async fn on_layout_updated(
        &mut self,
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    ) -> Result<()> {
        match Layout::get(self.conn.clone(), &service, &menu).await {
            Ok(item) => self.stream_map.emit(DBusEvent::LayoutReceived {
                item,
                service: Arc::clone(&service),
            })?,
            Err(err) => {
                log::error!("failed to get layout of {service}: {err:?}");
                return Ok(());
            }
        }

        Ok(())
    }

    async fn on_items_properties_updated(
        &mut self,
        service: Arc<str>,
        menu: Arc<OwnedObjectPath>,
    ) -> Result<()> {
        self.stream_map
            .emit(DBusEvent::LayoutUpdated { service, menu })
    }

    async fn on_layout_received(&mut self, service: Arc<str>, item: TrayItem) -> Result<()> {
        if let Some(event) = self.store.update_item(service, item) {
            self.tx.send(event)?;
        }
        Ok(())
    }
}

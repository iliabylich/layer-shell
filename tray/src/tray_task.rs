use std::sync::Arc;

use crate::{
    TrayEvent, TrayItem,
    dbus::{NameLostEvent, NameOwnerChangedEvent},
    dbus_event::DBusEvent,
    dbusmenu::{ItemsPropertiesUpdated, Layout, LayoutUpdated},
    status_notifier_item::{IconNameUpdated, IconPixmapUpdate, MenuUpdated},
    status_notifier_watcher::StatusNotifierWatcher,
    stream_id::StreamId,
    stream_map::StreamMap,
};
use anyhow::Result;
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;
use zbus::{Connection, zvariant::OwnedObjectPath};

pub(crate) struct TrayTask {
    stream_map: StreamMap,
    token: CancellationToken,
    conn: Connection,
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
                self.on_service_removed(&name);
            }
            DBusEvent::NameOwnerChanged { name, new_owner } => {
                if new_owner.is_none() {
                    self.on_service_removed(&name);
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

            DBusEvent::LayoutUpdated {
                service,
                menu,
                parent_id,
            } => {
                self.on_layout_updated(service, menu, parent_id).await?;
            }
            DBusEvent::ItemsPropertiesUpdated { service, menu } => {
                self.on_items_properties_updated(service, menu).await?;
            }

            DBusEvent::LayoutReceived {
                service,
                parent_id,
                item,
            } => {
                self.on_layout_received(service, parent_id, item).await?;
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

    fn on_service_removed(&mut self, service: &str) {
        let count_removed = self.stream_map.remove_service(&service);
        if count_removed > 0 {
            log::info!(target: "Tray", "lost service {service}, removed {count_removed} streams");
        }
    }

    async fn on_icon_name_changed(&mut self, service: Arc<str>, icon_name: String) -> Result<()> {
        log::info!("icon name changed: {service} {icon_name}");
        Ok(())
    }

    async fn on_icon_pixmap_changed(
        &mut self,
        service: Arc<str>,
        width: i32,
        height: i32,
        bytes: Vec<u8>,
    ) -> Result<()> {
        log::info!(
            "icon pixmap changed: {service} {width}x{height} ({} bytes)",
            bytes.len()
        );
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
        parent_id: i32,
    ) -> Result<()> {
        match Layout::get(self.conn.clone(), &service, &menu, parent_id).await {
            Ok(item) => self.stream_map.emit(DBusEvent::LayoutReceived {
                parent_id: 0,
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
        self.stream_map.emit(DBusEvent::LayoutUpdated {
            service,
            menu,
            parent_id: 0,
        })
    }

    async fn on_layout_received(
        &mut self,
        service: Arc<str>,
        parent_id: i32,
        item: TrayItem,
    ) -> Result<()> {
        log::info!("layout received on {service}/{parent_id} {item:?}");

        Ok(())
    }
}

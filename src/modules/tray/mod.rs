use crate::{
    dbus::{
        register_org_kde_status_notifier_watcher,
        tray::{DBusMenu, DBusNameOwnerChanged, StatusNotifierItem, UUID},
        ComCanonicalDbusmenuItemsPropertiesUpdated, ComCanonicalDbusmenuLayoutUpdated,
        OrgKdeStatusNotifierItemNewAttentionIcon, OrgKdeStatusNotifierItemNewIcon,
        OrgKdeStatusNotifierItemNewOverlayIcon, OrgKdeStatusNotifierItemNewStatus,
        OrgKdeStatusNotifierItemNewTitle, OrgKdeStatusNotifierItemNewToolTip,
    },
    epoll::{FdId, Reader},
    event::TrayApp,
    modules::{
        maybe_connected::MaybeConnected,
        tray::{item::Item, watcher::Watcher},
    },
    Event, VerboseSender,
};
use anyhow::{Context as _, Result};
use dbus::{
    arg::ReadAll,
    blocking::Connection,
    channel::{BusType, Channel},
    message::SignalArgs,
    Message, Path,
};
use dbus_crossroads::Crossroads;
use state::State;
use std::time::Duration;

mod item;
mod state;
mod watcher;

pub(crate) struct Tray {
    conn: Connection,
    state: State,
    cr: Crossroads,
    tx: VerboseSender<Event>,
}

impl Tray {
    fn try_new(tx: VerboseSender<Event>) -> Result<Self> {
        let mut channel =
            Channel::get_private(BusType::Session).context("failed to connect to DBus")?;
        channel.set_watch_enabled(true);
        let conn = Connection::from(channel);
        conn.request_name("org.kde.StatusNotifierWatcher", true, true, true)
            .context("failed to acquire DBus name")?;

        let state = State::new();

        let mut cr = Crossroads::new();
        let token = register_org_kde_status_notifier_watcher::<Watcher>(&mut cr);
        cr.insert("/StatusNotifierWatcher", &[token], Watcher::new());

        fn subscribe<T: SignalArgs + ReadAll>(conn: &Connection) {
            if let Err(err) = conn.add_match(T::match_rule(None, None), |_: T, _, _| true) {
                log::error!("Failed to subscribe to signal: {:?}", err);
            }
        }

        subscribe::<DBusNameOwnerChanged>(&conn);
        subscribe::<OrgKdeStatusNotifierItemNewAttentionIcon>(&conn);
        subscribe::<OrgKdeStatusNotifierItemNewIcon>(&conn);
        subscribe::<OrgKdeStatusNotifierItemNewOverlayIcon>(&conn);
        subscribe::<OrgKdeStatusNotifierItemNewStatus>(&conn);
        subscribe::<OrgKdeStatusNotifierItemNewTitle>(&conn);
        subscribe::<OrgKdeStatusNotifierItemNewToolTip>(&conn);
        subscribe::<ComCanonicalDbusmenuItemsPropertiesUpdated>(&conn);
        subscribe::<ComCanonicalDbusmenuLayoutUpdated>(&conn);

        Ok(Self {
            conn,
            state,
            cr,
            tx,
        })
    }

    pub(crate) fn new(tx: VerboseSender<Event>) -> MaybeConnected<Self> {
        MaybeConnected::new(Self::try_new(tx))
    }

    fn process_message(&mut self, message: Message) -> Result<()> {
        let service_id = message.sender().map(|b| b.to_string());

        if let Some(e) = DBusNameOwnerChanged::from_message(&message) {
            if e.name == e.old_owner && e.new_owner.is_empty() {
                let removed_service = e.name;
                let event = self.state.app_removed(removed_service);
                self.tx.send(event);
            }
        } else if OrgKdeStatusNotifierItemNewAttentionIcon::from_message(&message).is_some()
            || OrgKdeStatusNotifierItemNewIcon::from_message(&message).is_some()
            || OrgKdeStatusNotifierItemNewOverlayIcon::from_message(&message).is_some()
            || OrgKdeStatusNotifierItemNewStatus::from_message(&message).is_some()
            || OrgKdeStatusNotifierItemNewTitle::from_message(&message).is_some()
            || OrgKdeStatusNotifierItemNewToolTip::from_message(&message).is_some()
            || ComCanonicalDbusmenuItemsPropertiesUpdated::from_message(&message).is_some()
            || ComCanonicalDbusmenuLayoutUpdated::from_message(&message).is_some()
        {
            let service = message
                .sender()
                .context("failed to get sender")?
                .to_string();
            let updated_item = self
                .state
                .find(&service)
                .with_context(|| format!("failed to find service {service}"))?;
            self.reload_tray_app(updated_item)?;
        } else if self.cr.handle_message(message, &self.conn).is_ok() {
            if let Some(watcher) = self.cr.data_mut::<Watcher>(
                &Path::new("/StatusNotifierWatcher")
                    .ok()
                    .context("invalid path")?,
            ) {
                if let Some(service) = watcher.pop_new_item() {
                    let path = String::from("/StatusNotifierItem");
                    let menu_path = StatusNotifierItem::new(&service, &path).menu(&self.conn)?;

                    let item = Item {
                        service,
                        path,
                        menu_path,
                        service_id: service_id.unwrap_or_else(|| "unknown".to_string()),
                    };

                    self.reload_tray_app(item)?;
                }
            }
        }

        Ok(())
    }

    fn reload_tray_app(&mut self, item: Item) -> Result<()> {
        let dbus_menu = DBusMenu::new(&item.service, &item.menu_path);

        let root_item = dbus_menu.get_layout(&self.conn)?;
        let status_notifier_item = StatusNotifierItem::new(&item.service, &item.path);

        let app = TrayApp {
            root_item,
            icon: status_notifier_item.any_icon(&self.conn),
        };

        let event = self.state.app_added(item, app);
        self.tx.send(event);

        Ok(())
    }

    pub(crate) fn trigger(&mut self, uuid: String) -> Result<()> {
        let (service, path, id) = UUID::decode(uuid)?;
        DBusMenu::new(service, path).event(&self.conn, id)?;
        self.read()?;
        Ok(())
    }
}

impl Reader for Tray {
    type Output = ();

    const NAME: &str = "Tray";

    fn read(&mut self) -> Result<Self::Output> {
        while let Ok(Some(message)) = self
            .conn
            .channel()
            .blocking_pop_message(Duration::from_secs(0))
        {
            self.process_message(message)?;
        }

        Ok(())
    }

    fn fd(&self) -> i32 {
        self.conn.channel().watch().fd
    }

    fn fd_id(&self) -> FdId {
        FdId::TrayDBus
    }
}

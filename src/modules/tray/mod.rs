use crate::{
    channel::EventSender0,
    dbus::{
        ComCanonicalDbusmenuItemsPropertiesUpdated, ComCanonicalDbusmenuLayoutUpdated,
        OrgKdeStatusNotifierItemNewAttentionIcon, OrgKdeStatusNotifierItemNewIcon,
        OrgKdeStatusNotifierItemNewOverlayIcon, OrgKdeStatusNotifierItemNewStatus,
        OrgKdeStatusNotifierItemNewTitle, OrgKdeStatusNotifierItemNewToolTip,
        register_org_kde_status_notifier_watcher,
        tray::{DBusMenu, DBusNameOwnerChanged, StatusNotifierItem, UUID},
    },
    event::TrayApp,
    fd_id::FdId,
    modules::{
        Module,
        tray::{item::Item, watcher::Watcher},
    },
};
use anyhow::{Context as _, Result};
use dbus::{
    Message, Path,
    arg::ReadAll,
    blocking::Connection,
    channel::{BusType, Channel},
    message::SignalArgs,
};
use dbus_crossroads::Crossroads;
use state::State;
use std::{
    os::fd::{AsRawFd, RawFd},
    time::Duration,
};

mod item;
mod state;
mod watcher;

pub(crate) struct Tray {
    conn: Connection,
    state: State,
    cr: Crossroads,
    tx: EventSender0,
}

impl Module for Tray {
    const FD_ID: FdId = FdId::TrayDBus;
    const NAME: &str = "Tray";

    type ReadOutput = ();

    fn new(tx: EventSender0) -> Result<Self> {
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

    fn read_events(&mut self) -> Result<()> {
        while let Ok(Some(message)) = self
            .conn
            .channel()
            .blocking_pop_message(Duration::from_secs(0))
        {
            self.process_message(message)?;
        }

        Ok(())
    }
}

impl Tray {
    fn process_message(&mut self, message: Message) -> Result<()> {
        let sender = message
            .sender()
            .map(|b| b.to_string())
            .context("message has no sender")?;

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
            let updated_item = self
                .state
                .find(&sender)
                .with_context(|| format!("failed to find service {sender}"))?;
            self.reload_tray_app(updated_item)?;
        } else if self.cr.handle_message(message, &self.conn).is_ok() {
            if let Some(watcher) = self.cr.data_mut::<Watcher>(
                &Path::new("/StatusNotifierWatcher")
                    .ok()
                    .context("invalid path")?,
            ) {
                if let Some(maybe_path) = watcher.pop_new_item() {
                    let path = if maybe_path.starts_with('/') {
                        maybe_path
                    } else {
                        String::from("/StatusNotifierItem")
                    };
                    let menu_path = StatusNotifierItem::new(&sender, &path).menu(&self.conn)?;

                    let item = Item {
                        id: sender,
                        path,
                        menu_path,
                    };

                    self.reload_tray_app(item)?;
                }
            }
        }

        Ok(())
    }

    fn reload_tray_app(&mut self, item: Item) -> Result<()> {
        let dbus_menu = DBusMenu::new(&item.id, &item.menu_path);

        let root_item = dbus_menu.get_layout(&self.conn)?;
        let status_notifier_item = StatusNotifierItem::new(&item.id, &item.path);

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
        self.read_events()?;
        Ok(())
    }
}

impl AsRawFd for Tray {
    fn as_raw_fd(&self) -> RawFd {
        self.conn.channel().watch().fd
    }
}

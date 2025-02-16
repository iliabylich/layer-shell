use crate::{
    dbus::{
        register_org_kde_status_notifier_watcher,
        tray::{DBusMenu, DBusNameOwnerChanged, StatusNotifierItem, UUID},
        OrgKdeStatusNotifierItemNewAttentionIcon, OrgKdeStatusNotifierItemNewIcon,
        OrgKdeStatusNotifierItemNewOverlayIcon, OrgKdeStatusNotifierItemNewStatus,
        OrgKdeStatusNotifierItemNewTitle, OrgKdeStatusNotifierItemNewToolTip,
    },
    event::{TrayApp, TrayItem},
    modules::tray::{item::Item, watcher::Watcher},
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

pub(crate) struct ConnectedTray {
    conn: Connection,
    state: State,
    cr: Crossroads,
    tx: VerboseSender<Event>,
}

impl ConnectedTray {
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

        let this = Self {
            conn,
            state,
            cr,
            tx,
        };

        this.subscribe::<DBusNameOwnerChanged>();
        this.subscribe::<OrgKdeStatusNotifierItemNewAttentionIcon>();
        this.subscribe::<OrgKdeStatusNotifierItemNewIcon>();
        this.subscribe::<OrgKdeStatusNotifierItemNewOverlayIcon>();
        this.subscribe::<OrgKdeStatusNotifierItemNewStatus>();
        this.subscribe::<OrgKdeStatusNotifierItemNewTitle>();
        this.subscribe::<OrgKdeStatusNotifierItemNewToolTip>();

        Ok(this)
    }

    fn subscribe<T: SignalArgs + ReadAll>(&self) {
        if let Err(err) = self
            .conn
            .add_match(T::match_rule(None, None), |_: T, _, _| true)
        {
            log::error!("Failed to subscribe to signal: {:?}", err);
        }
    }

    fn read(&mut self) -> Result<()> {
        while let Ok(Some(message)) = self
            .conn
            .channel()
            .blocking_pop_message(Duration::from_secs(0))
        {
            self.process_message(message)?;
        }

        Ok(())
    }

    fn process_message(&mut self, message: Message) -> Result<()> {
        let sender = message.sender().map(|s| s.into_static());

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
        {
            let service = message
                .sender()
                .context("failed to get sender")?
                .to_string();
            let updated_item = self
                .state
                .find(&service)
                .context("failed to find service")?;
            self.reload_tray_app(updated_item)?;
        } else if self.cr.handle_message(message, &self.conn).is_ok() {
            if let Some(watcher) = self.cr.data_mut::<Watcher>(
                &Path::new("/StatusNotifierWatcher")
                    .ok()
                    .context("invalid path")?,
            ) {
                if let Some(mut path) = watcher.pop_new_item() {
                    let service = sender.context("failed to get sender")?.to_string();

                    let menu_path = if service == path {
                        path = String::from("/StatusNotifierItem");
                        String::from("/com/canonical/dbusmenu")
                    } else {
                        StatusNotifierItem::new(&service, &path).menu(&self.conn)?
                    };

                    let item = Item {
                        service,
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
        let dbus_menu = DBusMenu::new(&item.service, &item.menu_path);

        let items = dbus_menu.get_layout(&self.conn)?;
        let status_notifier_item = StatusNotifierItem::new(&item.service, &item.path);

        let app = TrayApp {
            items: items
                .into_iter()
                .map(|item| TrayItem {
                    label: item.label.into(),
                    disabled: item.disabled,
                    uuid: item.uuid.into(),
                })
                .collect::<Vec<_>>()
                .into(),
            icon: status_notifier_item.any_icon(&self.conn),
        };

        let event = self.state.app_added(item, app);
        self.tx.send(event);

        Ok(())
    }

    fn trigger(&mut self, uuid: String) -> Result<()> {
        let (service, path, id) = UUID::decode(uuid)?;
        DBusMenu::new(service, path).event(&self.conn, id)?;
        Ok(())
    }
}

pub(crate) enum Tray {
    Connected(ConnectedTray),
    Disconnected,
}

impl Tray {
    pub(crate) fn new(tx: VerboseSender<Event>) -> Self {
        ConnectedTray::try_new(tx)
            .inspect_err(|err| log::error!("{:?}", err))
            .map(Self::Connected)
            .unwrap_or(Self::Disconnected)
    }

    pub(crate) fn read(&mut self) {
        if let Self::Connected(inner) = self {
            if let Err(err) = inner.read() {
                log::error!("{:?}", err);
            }
        }
    }

    pub(crate) fn trigger(&mut self, uuid: String) {
        if let Self::Connected(inner) = self {
            if let Err(err) = inner.trigger(uuid) {
                log::error!("{:?}", err);
            }
        }
    }

    pub(crate) fn fd(&self) -> Option<i32> {
        match self {
            Self::Connected(ConnectedTray { conn, .. }) => Some(conn.channel().watch().fd),
            Self::Disconnected => None,
        }
    }
}

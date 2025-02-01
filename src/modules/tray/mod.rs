use crate::{
    dbus::{
        register_org_kde_status_notifier_watcher,
        tray::{DBusMenu, DBusNameOwnerChanged, StatusNotifierItem, UUID},
        OrgKdeStatusNotifierItemNewAttentionIcon, OrgKdeStatusNotifierItemNewIcon,
        OrgKdeStatusNotifierItemNewOverlayIcon, OrgKdeStatusNotifierItemNewStatus,
        OrgKdeStatusNotifierItemNewTitle, OrgKdeStatusNotifierItemNewToolTip,
    },
    event::{TrayApp, TrayItem},
    modules::tray::{
        channel::{TrayCommand, CHANNEL},
        item::Item,
        watcher::Watcher,
    },
    scheduler::{Module, RepeatingModule},
    Command,
};
use anyhow::{Context as _, Result};
use dbus::{
    arg::ReadAll, blocking::Connection, channel::MatchingReceiver as _, message::SignalArgs,
    Message,
};
use dbus_crossroads::Crossroads;
use state::State;
use std::time::Duration;

mod channel;
mod item;
mod state;
mod watcher;

pub(crate) struct Tray {
    conn: Connection,
    state: State,
}

impl Module for Tray {
    const NAME: &str = "Tray";

    fn start() -> Result<Option<Box<dyn RepeatingModule>>> {
        let conn = Connection::new_session()?;
        let state = State::new();

        conn.add_match(
            DBusNameOwnerChanged::match_rule(None, None),
            |e: DBusNameOwnerChanged, _, _| {
                if e.name == e.old_owner && e.new_owner.is_empty() {
                    CHANNEL.emit(TrayCommand::Removed { service: e.name });
                }
                true
            },
        )
        .context("failed to subscribe to NameOwnerChanged signal")?;

        subscribe_to_item_update::<OrgKdeStatusNotifierItemNewAttentionIcon>(&conn)?;
        subscribe_to_item_update::<OrgKdeStatusNotifierItemNewIcon>(&conn)?;
        subscribe_to_item_update::<OrgKdeStatusNotifierItemNewOverlayIcon>(&conn)?;
        subscribe_to_item_update::<OrgKdeStatusNotifierItemNewStatus>(&conn)?;
        subscribe_to_item_update::<OrgKdeStatusNotifierItemNewTitle>(&conn)?;
        subscribe_to_item_update::<OrgKdeStatusNotifierItemNewToolTip>(&conn)?;

        conn.request_name("org.kde.StatusNotifierWatcher", true, true, true)?;

        let mut cr = Crossroads::new();
        let token = register_org_kde_status_notifier_watcher::<Watcher>(&mut cr);

        cr.insert("/StatusNotifierWatcher", &[token], Watcher);

        conn.start_receive(
            dbus::message::MatchRule::new_method_call(),
            Box::new(move |msg, conn| {
                if cr.handle_message(msg, conn).is_err() {
                    log::error!("Failed to handle message");
                }
                true
            }),
        );

        Ok(Some(Box::new(Tray { conn, state })))
    }
}

impl RepeatingModule for Tray {
    fn tick(&mut self) -> Result<Duration> {
        while let Some(command) = CHANNEL.try_recv() {
            if let Err(err) = handle_command(&self.conn, command, &mut self.state) {
                log::error!("{:?}", err);
            }
        }

        while self.conn.process(Duration::from_millis(100))? {}

        Ok(Duration::from_millis(200))
    }

    fn exec(&mut self, cmd: &Command) -> Result<()> {
        if let Command::TriggerTray { uuid } = cmd {
            let uuid = String::from(uuid.clone());
            let (service, path, id) = UUID::decode(uuid)?;
            DBusMenu::new(service, path).event(&self.conn, id)?;
        }
        Ok(())
    }
}

fn subscribe_to_item_update<T: ReadAll + SignalArgs + 'static>(conn: &Connection) -> Result<()> {
    let _token = conn
        .add_match(
            T::match_rule(None, None),
            |_: T, _: &Connection, message: &Message| {
                if let Some(service) = message.sender() {
                    CHANNEL.emit(TrayCommand::Updated {
                        service: service.to_string(),
                    });
                }
                true
            },
        )
        .context("failed to call AddMatch")?;
    Ok(())
}

fn handle_command(conn: &Connection, command: TrayCommand, state: &mut State) -> Result<()> {
    match command {
        TrayCommand::Added { service, mut path } => {
            let menu_path;

            if service == path {
                path = String::from("/StatusNotifierItem");
                menu_path = String::from("/com/canonical/dbusmenu")
            } else {
                menu_path = StatusNotifierItem::new(&service, &path).menu(conn)?;
            }
            let item = Item {
                service,
                path,
                menu_path,
            };

            reload_tray_app(conn, item, state)?;
        }

        TrayCommand::Removed { service } => {
            state.app_removed(service);
        }

        TrayCommand::Updated { service } => {
            let item = state
                .find(&service)
                .with_context(|| format!("failed to find service {service}"))?;
            reload_tray_app(conn, item, state)?;
        }
    }

    Ok(())
}

fn reload_tray_app(conn: &Connection, item: Item, state: &mut State) -> Result<()> {
    let dbus_menu = DBusMenu::new(&item.service, &item.menu_path);

    let items = dbus_menu.get_layout(conn)?;
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
        icon: status_notifier_item.any_icon(conn),
    };

    state.app_added(item, app);

    Ok(())
}

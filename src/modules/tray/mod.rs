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
        channel::{Command, CHANNEL},
        item::Item,
        watcher::Watcher,
    },
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

pub(crate) fn setup() {
    std::thread::spawn(|| {
        if let Err(err) = try_setup() {
            log::error!("Failed to become StatusNotifierWatcher: {:?}", err);
        }
    });
}

pub(crate) fn trigger(uuid: *const u8) {
    let uuid = unsafe { std::ffi::CStr::from_ptr(uuid.cast()) };
    let Ok(uuid) = uuid.to_str() else {
        log::error!("Tray: invalid uuid");
        return;
    };

    CHANNEL.emit(Command::TriggerItem {
        uuid: uuid.to_string(),
    });
}

fn try_setup() -> Result<()> {
    let conn = Connection::new_session()?;
    conn.add_match(
        DBusNameOwnerChanged::match_rule(None, None),
        |e: DBusNameOwnerChanged, _, _| {
            if e.name == e.old_owner && e.new_owner.is_empty() {
                CHANNEL.emit(Command::ServiceRemoved { service: e.name });
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

    loop {
        while let Some(command) = CHANNEL.try_recv() {
            if let Err(err) = handle_command(&conn, command) {
                log::error!("{:?}", err);
            }
        }

        conn.process(Duration::from_millis(200))?;
    }
}

fn subscribe_to_item_update<T: ReadAll + SignalArgs + 'static>(conn: &Connection) -> Result<()> {
    let _token = conn
        .add_match(
            T::match_rule(None, None),
            |_: T, _: &Connection, message: &Message| {
                if let Some(service) = message.sender() {
                    CHANNEL.emit(Command::ServiceUpdated {
                        service: service.to_string(),
                    });
                }
                true
            },
        )
        .context("failed to call AddMatch")?;
    Ok(())
}

fn handle_command(conn: &Connection, command: Command) -> Result<()> {
    match command {
        Command::ServiceAdded { service, mut path } => {
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

            reload_tray_app(conn, item)?;
        }

        Command::ServiceRemoved { service } => {
            State::app_removed(service);
        }

        Command::ServiceUpdated { service } => {
            let item = State::find(&service)
                .with_context(|| format!("failed to find service {service}"))?;
            reload_tray_app(conn, item)?;
        }

        Command::TriggerItem { uuid } => {
            let (service, path, id) = UUID::decode(uuid)?;
            DBusMenu::new(service, path).event(conn, id)?;
        }
    }

    Ok(())
}

fn reload_tray_app(conn: &Connection, item: Item) -> Result<()> {
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

    state::State::app_added(item, app);

    Ok(())
}

use crate::{
    dbus::{
        register_org_kde_status_notifier_watcher,
        tray::{DBusMenu, DBusNameOwnerChanged, StatusNotifierItem, UUID},
    },
    modules::tray::{
        channel::{Command, CHANNEL},
        watcher::Watcher,
    },
};
use anyhow::{Context as _, Result};
use dbus::{blocking::Connection, channel::MatchingReceiver as _, message::SignalArgs};
use dbus_crossroads::Crossroads;
use std::time::Duration;

mod channel;
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
            match command {
                Command::TriggerItem { uuid } => {
                    if let Ok((service, path, id)) = UUID::decode(uuid) {
                        if let Err(err) = DBusMenu::new(service, path).event(&conn, id) {
                            log::error!("{:?}", err);
                        }
                    }
                }

                Command::ServiceAdded { service, mut path } => {
                    let dbus_menu_path;

                    if service == path {
                        path = String::from("/StatusNotifierItem");
                        dbus_menu_path = String::from("/com/canonical/dbusmenu")
                    } else {
                        match StatusNotifierItem::new(&service, &path).menu(&conn) {
                            Ok(menu_path) => dbus_menu_path = menu_path.to_string(),
                            Err(err) => {
                                log::error!("{:?}", err);
                                continue;
                            }
                        }
                    }

                    let dbus_menu = DBusMenu::new(&service, &dbus_menu_path);

                    match dbus_menu.get_layout(&conn) {
                        Ok(mut app) => {
                            let status_notifier_item = StatusNotifierItem::new(&service, &path);
                            app.icon = status_notifier_item.any_icon(&conn);

                            state::State::app_added(service, app)
                        }
                        Err(err) => log::error!("{:?}", err),
                    }
                }

                Command::ServiceRemoved { service } => {
                    state::State::app_removed(service);
                }

                _ => todo!("{:?}", command),
            }
        }

        conn.process(Duration::from_millis(200))?;
    }
}

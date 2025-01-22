use crate::{
    dbus::{
        register_org_kde_status_notifier_watcher,
        tray::{DBusMenu, DBusNameOwnerChanged, StatusNotifierItem, UUID},
    },
    lock_channel::LockChannel,
    modules::tray::watcher::{Watcher, WatcherData},
};
use anyhow::{Context as _, Result};
use dbus::{blocking::Connection, channel::MatchingReceiver as _, message::MatchRule};
use dbus_crossroads::Crossroads;
use std::{sync::LazyLock, time::Duration};

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

    CHANNEL.emit(uuid.to_string());
}

static CHANNEL: LazyLock<LockChannel<String>> = LazyLock::new(LockChannel::new);

fn try_setup() -> Result<()> {
    let conn = Connection::new_session()?;
    conn.add_match(
        MatchRule::new_signal(
            <DBusNameOwnerChanged as dbus::message::SignalArgs>::INTERFACE,
            <DBusNameOwnerChanged as dbus::message::SignalArgs>::NAME,
        ),
        |signal: DBusNameOwnerChanged, _conn, _message| {
            if signal.is_remove() {
                state::State::app_removed(signal.name);
            }
            true
        },
    )
    .context("failed to subscribe to NameOwnerChanged signal")?;

    conn.request_name("org.kde.StatusNotifierWatcher", true, true, true)?;

    let mut cr = Crossroads::new();
    let token = register_org_kde_status_notifier_watcher::<Watcher>(&mut cr);

    let (tx, rx) = std::sync::mpsc::channel();
    let watcher = Watcher::new(tx);

    cr.insert("/StatusNotifierWatcher", &[token], watcher);

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
        while let Some(uuid) = CHANNEL.try_recv() {
            if let Ok((service, path, id)) = UUID::decode(uuid) {
                if let Err(err) = DBusMenu::new(service, path).event(&conn, id) {
                    log::error!("{:?}", err);
                }
            }
        }

        conn.process(Duration::from_millis(200))?;

        while let Ok(data) = rx.try_recv() {
            let (service, item_path, dbus_menu_path) = match data {
                WatcherData::StatusNotifierItem { service, path } => {
                    match StatusNotifierItem::new(&service, &path).menu(&conn) {
                        Ok(dbus_menu_path) => (service, path, dbus_menu_path.to_string()),
                        Err(err) => {
                            log::error!("{:?}", err);
                            continue;
                        }
                    }
                }
                WatcherData::CanonicalDBusMenu { service } => (
                    service,
                    String::from("/StatusNotifierItem"),
                    String::from("/com/canonical/dbusmenu"),
                ),
            };

            let dbus_menu = DBusMenu::new(&service, &dbus_menu_path);

            match dbus_menu.get_layout(&conn) {
                Ok(mut app) => {
                    let status_notifier_item = StatusNotifierItem::new(&service, item_path);
                    app.icon = status_notifier_item.any_icon(&conn);

                    state::State::app_added(service, app)
                }
                Err(err) => log::error!("{:?}", err),
            }
        }
    }
}

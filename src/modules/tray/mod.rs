use crate::{
    dbus::{
        register_org_kde_status_notifier_watcher,
        tray::{DBusMenu, DBusNameOwnerChanged, StatusNotifierItem, UUID},
        OrgKdeStatusNotifierItemNewAttentionIcon, OrgKdeStatusNotifierItemNewIcon,
        OrgKdeStatusNotifierItemNewOverlayIcon, OrgKdeStatusNotifierItemNewStatus,
        OrgKdeStatusNotifierItemNewTitle, OrgKdeStatusNotifierItemNewToolTip,
    },
    event::{TrayApp, TrayItem},
    modules::tray::{channel::TrayCommand, item::Item, watcher::Watcher},
    scheduler::Actor,
    Command, Event,
};
use anyhow::{Context as _, Result};
use dbus::{
    arg::ReadAll, blocking::Connection, channel::MatchingReceiver as _, message::SignalArgs,
    Message,
};
use dbus_crossroads::Crossroads;
use state::State;
use std::{
    ops::ControlFlow,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

mod channel;
mod item;
mod state;
mod watcher;

pub(crate) struct Tray {
    conn: Connection,
    state: State,
    rx: Receiver<TrayCommand>,
}

impl Actor for Tray {
    fn name() -> &'static str {
        "Tray"
    }

    fn start(tx: Sender<Event>) -> Result<Box<dyn Actor>> {
        let conn = Connection::new_session()?;
        let state = State::new(tx);

        let (itx, irx) = std::sync::mpsc::channel::<TrayCommand>();

        {
            let itx = itx.clone();
            conn.add_match(
                DBusNameOwnerChanged::match_rule(None, None),
                move |e: DBusNameOwnerChanged, _, _| {
                    if e.name == e.old_owner && e.new_owner.is_empty() {
                        if let Err(err) = itx.send(TrayCommand::Removed { service: e.name }) {
                            log::error!("failed to send TrayCommand: {:?}", err);
                        }
                    }
                    true
                },
            )
            .context("failed to subscribe to NameOwnerChanged signal")?;
        }

        subscribe_to_item_update::<OrgKdeStatusNotifierItemNewAttentionIcon>(&conn, &itx)?;
        subscribe_to_item_update::<OrgKdeStatusNotifierItemNewIcon>(&conn, &itx)?;
        subscribe_to_item_update::<OrgKdeStatusNotifierItemNewOverlayIcon>(&conn, &itx)?;
        subscribe_to_item_update::<OrgKdeStatusNotifierItemNewStatus>(&conn, &itx)?;
        subscribe_to_item_update::<OrgKdeStatusNotifierItemNewTitle>(&conn, &itx)?;
        subscribe_to_item_update::<OrgKdeStatusNotifierItemNewToolTip>(&conn, &itx)?;

        conn.request_name("org.kde.StatusNotifierWatcher", true, true, true)?;

        let mut cr = Crossroads::new();
        let token = register_org_kde_status_notifier_watcher::<Watcher>(&mut cr);

        cr.insert("/StatusNotifierWatcher", &[token], Watcher { tx: itx });

        conn.start_receive(
            dbus::message::MatchRule::new_method_call(),
            Box::new(move |msg, conn| {
                if cr.handle_message(msg, conn).is_err() {
                    log::error!("Failed to handle message");
                }
                true
            }),
        );

        Ok(Box::new(Tray {
            conn,
            state,
            rx: irx,
        }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        while let Ok(command) = self.rx.try_recv() {
            if let Err(err) = handle_command(&self.conn, command, &mut self.state) {
                log::error!("{:?}", err);
            }
        }

        while self.conn.process(Duration::from_millis(0))? {}

        Ok(ControlFlow::Continue(Duration::from_millis(200)))
    }

    fn exec(&mut self, cmd: &Command) -> Result<ControlFlow<()>> {
        if let Command::TriggerTray { uuid } = cmd {
            let (service, path, id) = UUID::decode(uuid)?;
            DBusMenu::new(service, path).event(&self.conn, id)?;
        }
        Ok(ControlFlow::Continue(()))
    }
}

impl std::fmt::Debug for Tray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tray")
            .field("conn", &"<conn>")
            .field("state", &self.state)
            .finish()
    }
}

fn subscribe_to_item_update<T: ReadAll + SignalArgs + 'static>(
    conn: &Connection,
    tx: &Sender<TrayCommand>,
) -> Result<()> {
    let tx = tx.clone();

    let _token = conn
        .add_match(
            T::match_rule(None, None),
            move |_: T, _: &Connection, message: &Message| {
                if let Some(service) = message.sender() {
                    if let Err(err) = tx.send(TrayCommand::Updated {
                        service: service.to_string(),
                    }) {
                        log::error!("failed to send TrayCommand::Update: {:?}", err);
                    }
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

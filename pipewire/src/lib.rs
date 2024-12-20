#![allow(clippy::type_complexity)]

use std::{
    rc::Rc,
    sync::mpsc::{Receiver, Sender},
};

use anyhow::{Context as _, Result};
use pipewire::{
    context::Context,
    loop_::TimerSource,
    main_loop::MainLoop,
    metadata::Metadata,
    node::Node,
    registry::{Listener, Registry},
    spa::param::ParamType,
};

mod command;
mod event;
mod nodes;
mod store;

pub use command::Command;
pub use event::{Event, Volume};
use store::Store;

pub fn start() -> (Sender<Command>, Receiver<Event>) {
    let (etx, erx) = std::sync::mpsc::channel::<Event>();
    let (ctx, crx) = std::sync::mpsc::channel::<Command>();

    std::thread::spawn(move || {
        if let Err(err) = start_pw_communication(etx, crx) {
            log::error!("{}", err);
        }
    });

    (ctx, erx)
}

fn start_pw_communication(tx: Sender<Event>, rx: Receiver<Command>) -> Result<()> {
    let mainloop = MainLoop::new(None).context("failed to instantiate PW loop")?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;

    let registry = Rc::new(core.get_registry()?);

    let store = Store::new();

    let _timer = start_polling_commands(&mainloop, Store::shallow_clone(&store), rx);
    let _listener = start_pw_listener(Rc::clone(&registry), Store::shallow_clone(&store), tx);

    mainloop.run();

    Ok(())
}

fn start_polling_commands(
    mainloop: &MainLoop,
    store: Store,
    rx: Receiver<Command>,
) -> TimerSource<'_> {
    let timer = mainloop.loop_().add_timer(move |_| {
        if let Ok(command) = rx.try_recv() {
            command.dispatch(&store)
        }
    });
    timer.update_timer(
        Some(std::time::Duration::from_millis(1)),
        Some(std::time::Duration::from_millis(50)),
    );
    timer
}

fn start_pw_listener(registry: Rc<Registry>, store: Store, tx: Sender<Event>) -> Listener {
    let registry_weak = Rc::downgrade(&registry);

    registry
        .add_listener_local()
        .global(move |obj| {
            let id = obj.id;
            let tx = tx.clone();

            let Some(registry) = registry_weak.upgrade() else {
                log::error!("Registry reference is no longer alive");
                return;
            };

            if nodes::metadata::is_default(obj) {
                let Ok(meta) = registry.bind::<Metadata, _>(obj) else {
                    log::error!("failed to bind to Metadata object");
                    return;
                };

                let listener = {
                    let store = Store::shallow_clone(&store);
                    meta.add_listener_local()
                        .property(move |_, key, _, value| {
                            if let Some(name) =
                                nodes::metadata::parse_audio_sink_changed(key, value)
                            {
                                store.set_default_sink_name(name);
                            }
                            0
                        })
                        .register()
                };

                store.add_meta(id, meta);
                store.add_listener(id, Box::new(listener));
            } else if let Some(name) = nodes::sink::parse_name(obj) {
                let Ok(node) = registry.bind::<Node, _>(obj) else {
                    log::error!("failed to bind to Metadata object");
                    return;
                };

                node.subscribe_params(&[ParamType::Props]);
                let listener = node
                    .add_listener_local()
                    .param(move |_, _, _, _, param| {
                        if let Some(channels) = nodes::sink::parse_volume_changed_event(param) {
                            if let Err(err) = tx.send(Event::Volume(Volume {
                                volume: channels[0],
                            })) {
                                log::error!("failed to send event: {:?}", err);
                            }
                        }
                    })
                    .register();

                store.add_node(id, node);
                store.add_listener(id, Box::new(listener));
                store.add_sink_name(name, id);
            }
        })
        .register()
}

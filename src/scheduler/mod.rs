use crate::{macros::fatal, Command, Event};
pub(crate) use actor::Actor;
pub(crate) use config::{ActorConfig, Config};
use queue::Queue;
use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    time::Duration,
};

mod actor;
mod config;
mod queue;
mod timer;

pub(crate) struct Scheduler {
    queue: Arc<Mutex<Queue>>,
    txs: HashMap<&'static str, Sender<Command>>,
    thread_pool: threadpool::ThreadPool,
    cmd_rx: Receiver<Command>,
}

impl Scheduler {
    pub(crate) fn new(config: Config, e_tx: Sender<Event>, cmd_rx: Receiver<Command>) -> Self {
        let mut queue = Queue::new();
        let mut txs = HashMap::new();
        let thread_pool = threadpool::ThreadPool::new(5);

        for config in config.into_iter() {
            let (tx, rx) = std::sync::mpsc::channel::<Command>();
            tx.send(Command::Probe)
                .unwrap_or_else(|_| fatal!("channel was immediately closed"));
            let ActorConfig { name, start } = config;

            log::info!("Starting module {name}");
            match start(e_tx.clone()) {
                Ok(module) => {
                    log::info!("Module {name} has successfully started");
                    queue.register(name, module, rx);
                    txs.insert(name, tx);
                }
                Err(err) => log::error!("Failed to start module {name}: {:?}", err),
            }
        }

        let queue = Arc::new(Mutex::new(queue));
        Self {
            queue,
            txs,
            thread_pool,
            cmd_rx,
        }
    }

    pub(crate) fn run(self) {
        let Self {
            queue,
            txs,
            thread_pool,
            cmd_rx,
        } = self;

        loop {
            while let Some((item, action)) = queue
                .lock()
                .unwrap_or_else(|_| fatal!("poisoned lock"))
                .pop_ready()
            {
                let queue = Arc::clone(&queue);
                thread_pool.execute(move || {
                    let mut item = item;

                    match action.run(
                        item.name,
                        &mut *item.module,
                        &mut item.tick_timer,
                        &mut item.exec_timer,
                        &item.rx,
                    ) {
                        Ok(_) => queue
                            .lock()
                            .unwrap_or_else(|_| fatal!("poisoned lock"))
                            .push(item),
                        Err(err) => {
                            log::error!("Stopping module {}: {:?}", item.name, err)
                        }
                    }
                });
            }

            while let Ok(cmd) = cmd_rx.try_recv() {
                for (name, tx) in txs.iter() {
                    if tx.send(cmd.clone()).is_err() {
                        log::error!(
                            "Failed to send command {:?} to module {}, channel is closed",
                            cmd,
                            name
                        );
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

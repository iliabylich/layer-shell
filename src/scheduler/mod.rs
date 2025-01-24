use crate::Command;
use anyhow::Result;
use queue::Queue;
use std::collections::HashMap;
use thread_pool::ThreadPool;

mod queue;
mod queue_item;
mod thread_pool;

pub(crate) trait Module {
    const NAME: &str;

    fn start() -> Result<Option<(u64, fn() -> Result<()>)>>;
}

pub(crate) struct Scheduler {
    modules: HashMap<&'static str, fn() -> Result<Option<(u64, fn() -> Result<()>)>>>,
    schedule: HashMap<&'static str, (u64, fn() -> Result<()>)>,
}

impl Scheduler {
    pub(crate) fn init() {
        Queue::init();
    }

    pub(crate) fn new() -> Self {
        Self {
            modules: HashMap::new(),
            schedule: HashMap::new(),
        }
    }

    pub(crate) fn add<T: Module>(&mut self) {
        let name = T::NAME;
        self.modules.insert(name, T::start);
    }

    fn process_queue(&self) {
        let ts = now();

        while let Some(name) = Queue::pop_min_lt(ts) {
            let (interval, tick_fn) = self
                .schedule
                .get(name)
                .copied()
                .expect("bug, module must be periodic");

            ThreadPool::execute_and_enqueue_again(name, tick_fn, interval);
        }

        while let Some(command) = Command::try_recv() {
            ThreadPool::execute_once(move || command.execute());
        }
    }

    pub(crate) fn start_loop(&mut self) {
        self.start_and_enqueue_repeating_modules();

        loop {
            self.process_queue();

            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    fn start_and_enqueue_repeating_modules(&mut self) {
        for (name, start_fn) in self.modules.iter() {
            log::info!("Starting module {name}");

            match start_fn() {
                Ok(tick) => {
                    log::info!("Module {name} has started");
                    match tick {
                        Some((interval, tick_fn)) => {
                            self.schedule.insert(name, (interval, tick_fn));

                            log::info!("Enqueueing initial tick of module {name}");
                            ThreadPool::execute_and_enqueue_again(name, tick_fn, interval);
                        }
                        None => log::info!("Module {name} doesn't tick"),
                    }
                }
                Err(err) => log::error!("Failed to start module {name}, NOT queueing: {:?}", err),
            }
        }
    }
}

fn now() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}

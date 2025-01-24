use crate::Command;
use anyhow::Result;
use queue::Queue;
use std::any::Any;
use thread_pool::ThreadPool;

mod queue;
mod queue_item;
mod thread_pool;

pub(crate) struct RepeatingModule {
    pub(crate) state: Box<dyn Any + Send + 'static>,
    pub(crate) interval_in_ms: u64,
    pub(crate) tick: fn(&mut Box<dyn Any + Send + 'static>) -> Result<()>,
}

pub(crate) trait Module {
    const NAME: &str;
    const INTERVAL: Option<u64>;

    fn start() -> Result<Box<dyn Any + Send + 'static>>;
    fn tick(_: &mut Box<dyn Any + Send + 'static>) -> Result<()>;
}

pub(crate) struct Scheduler {
    modules: Vec<(
        &'static str,
        Option<u64>,
        fn() -> Result<Box<dyn Any + Send + 'static>>,
        fn(&mut Box<dyn Any + Send + 'static>) -> Result<()>,
    )>,
    thread_pool: ThreadPool,
}

impl Scheduler {
    pub(crate) fn init() {
        Queue::init();
    }

    pub(crate) fn new() -> Self {
        Self {
            modules: vec![],
            thread_pool: ThreadPool::new(),
        }
    }

    pub(crate) fn add<T: Module>(&mut self) {
        self.modules.push((T::NAME, T::INTERVAL, T::start, T::tick))
    }

    fn process_queue(&self) {
        let ts = now();

        while let Some((name, module)) = Queue::pop_min_lt(ts) {
            self.thread_pool.execute_and_enqueue_again(name, module);
        }

        while let Some(command) = Command::try_recv() {
            self.thread_pool.execute_once(move || command.execute());
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
        for (name, interval_in_ms, start, tick) in std::mem::take(&mut self.modules) {
            log::info!("Starting module {name}");

            match start() {
                Ok(state) => {
                    log::info!("Module {name} has started");

                    if let Some(interval_in_ms) = interval_in_ms {
                        log::info!("Enqueueing initial tick of module {name}");
                        self.thread_pool.execute_and_enqueue_again(
                            name,
                            RepeatingModule {
                                state,
                                interval_in_ms,
                                tick,
                            },
                        );
                    } else {
                        log::info!("Module {name} doesn't tick");
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

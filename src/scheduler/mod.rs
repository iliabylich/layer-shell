use crate::Command;
use anyhow::Result;
use queue::Queue;
use std::time::Duration;
use thread_pool::ThreadPool;

mod queue;
mod queue_item;
mod thread_pool;

pub(crate) trait Module {
    const NAME: &str;

    fn start() -> Result<Option<Box<dyn RepeatingModule>>>;
}

pub(crate) trait RepeatingModule: Send {
    fn tick(&mut self) -> Result<Duration>;
    fn exec(&mut self, cmd: &Command) -> Result<()>;
}

pub(crate) struct Scheduler {
    modules_to_start: Vec<(
        &'static str,
        fn() -> Result<Option<Box<dyn RepeatingModule>>>,
    )>,
    thread_pool: ThreadPool,
}

impl Scheduler {
    pub(crate) fn init() {
        Queue::init();
    }

    pub(crate) fn new() -> Self {
        Self {
            modules_to_start: vec![],
            thread_pool: ThreadPool::new(),
        }
    }

    pub(crate) fn add<T: Module>(&mut self) {
        self.modules_to_start.push((T::NAME, T::start))
    }

    fn process_queue(&self) {
        while let Some((name, module)) = Queue::pop_ready() {
            self.thread_pool.execute_and_enqueue_again(name, module);
        }

        while let Some(cmd) = Command::try_recv() {
            self.thread_pool
                .execute_once(move || Queue::foreach(|module, _name| module.exec(&cmd)));
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
        for (name, start) in std::mem::take(&mut self.modules_to_start) {
            log::info!("Starting module {name}");

            match start() {
                Ok(repeat) => {
                    log::info!("Module {name} has started");

                    if let Some(repeat) = repeat {
                        log::info!("Enqueueing initial tick of module {name}");
                        self.thread_pool.execute_and_enqueue_again(name, repeat);
                    } else {
                        log::info!("Module {name} doesn't tick");
                    }
                }
                Err(err) => log::error!("Failed to start module {name}, NOT queueing: {:?}", err),
            }
        }
    }
}

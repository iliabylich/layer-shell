use anyhow::Result;

use crate::scheduler::{queue::Queue, RepeatingModule};

pub(crate) struct ThreadPool {
    inner: threadpool::ThreadPool,
}

impl ThreadPool {
    pub(crate) fn new() -> Self {
        Self {
            inner: threadpool::ThreadPool::new(5),
        }
    }
    pub(crate) fn execute_and_enqueue_again(
        &self,
        name: &'static str,
        mut module: Box<dyn RepeatingModule>,
    ) {
        self.inner.execute(move || match module.tick() {
            Ok(interval) => {
                let now = chrono::Utc::now().timestamp_millis() as u64;
                Queue::push(name, now + interval.as_millis() as u64, module);
            }
            Err(err) => log::error!("failed to tick {name} mod: {:?}", err),
        });
    }

    pub(crate) fn execute_once(&self, f: impl FnOnce() -> Result<()> + Send + 'static) {
        self.inner.execute(move || {
            if let Err(err) = f() {
                log::error!("{:?}", err);
            }
        });
    }
}

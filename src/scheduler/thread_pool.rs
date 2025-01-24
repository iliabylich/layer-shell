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
        mut module: RepeatingModule,
    ) {
        self.inner.execute(move || {
            if let Err(err) = (module.tick)(&mut module.state) {
                log::error!("failed to tick {name} mod: {:?}", err);
            }

            let now = chrono::Utc::now().timestamp_millis() as u64;
            Queue::push(name, now + module.interval_in_ms, module);
        });
    }

    pub(crate) fn execute_once(&self, f: impl FnOnce() + Send + 'static) {
        self.inner.execute(f);
    }
}

use crate::scheduler::queue::Queue;
use anyhow::Result;
use std::sync::LazyLock;

static THREAD_POOL: LazyLock<threadpool::ThreadPool> =
    LazyLock::new(|| threadpool::ThreadPool::new(5));

pub(crate) struct ThreadPool;

impl ThreadPool {
    pub(crate) fn execute_and_enqueue_again(
        name: &'static str,
        f: fn() -> Result<()>,
        interval: u64,
    ) {
        THREAD_POOL.execute(move || {
            if let Err(err) = f() {
                log::error!("failed to tick {name} mod: {:?}", err);
            }

            let now = chrono::Utc::now().timestamp_millis() as u64;
            Queue::push(name, now + interval);
        });
    }

    pub(crate) fn execute_once(f: impl FnOnce() + Send + 'static) {
        THREAD_POOL.execute(f);
    }
}

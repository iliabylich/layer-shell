use crate::utils::exec_async;

pub(crate) struct Memory {
    pub(crate) used: f64,
    pub(crate) total: f64,
}

impl Memory {
    pub(crate) fn subscribe<F>(on_change: F)
    where
        F: Fn(Memory) + 'static,
    {
        gtk4::glib::spawn_future_local(async move {
            loop {
                let value = Memory::parse().await;
                on_change(value);
                gtk4::glib::timeout_future_seconds(1).await;
            }
        });
    }

    async fn parse() -> Memory {
        let stdout = exec_async(&["free", "-m"]).await;
        let line = stdout.split("\n").nth(1).unwrap();
        let mut parts = line.split_ascii_whitespace().skip(1);
        let total = parts.next().unwrap();
        let used = parts.next().unwrap();

        Memory {
            used: used.parse::<f64>().unwrap() / 1024.0,
            total: total.parse::<f64>().unwrap() / 1024.0,
        }
    }
}

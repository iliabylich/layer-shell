use crate::utils::exec_async;

pub(crate) struct CPU {
    previous: Option<Vec<CpuCoreInfo>>,
}

impl CPU {
    pub(crate) fn spawn<F>(on_change: F)
    where
        F: Fn(Vec<usize>) + 'static,
    {
        gtk4::glib::spawn_future_local(async move {
            let mut cpu = CPU { previous: None };
            loop {
                let values = cpu.load().await;
                on_change(values);
                gtk4::glib::timeout_future_seconds(1).await;
            }
        });
    }

    async fn load(&mut self) -> Vec<usize> {
        let current = Self::parse().await;
        let count = current.len();

        if let Some(previous) = self.previous.take() {
            assert_eq!(previous.len(), current.len());

            let usage = previous
                .iter()
                .zip(current.iter())
                .map(|(prev, next)| next.load_comparing_to(prev))
                .collect::<Vec<_>>();

            self.previous = Some(current);

            usage
        } else {
            self.previous = Some(current);
            vec![0; count]
        }
    }

    async fn parse() -> Vec<CpuCoreInfo> {
        let stdout = exec_async(&["cat", "/proc/stat"]).await;
        stdout
            .split("\n")
            .filter(|line| line.starts_with("cpu") && line.as_bytes()[3].is_ascii_digit())
            .map(|line| CpuCoreInfo::parse(line))
            .collect()
    }
}

pub(crate) struct CpuCoreInfo {
    id: usize,
    idle: usize,
    total: usize,
}

impl CpuCoreInfo {
    fn load_comparing_to(&self, previous: &Self) -> usize {
        assert_eq!(self.id, previous.id);

        let idle_d = (self.idle - previous.idle) as f64;
        let total_d = (self.total - previous.total) as f64;

        (100.0 * (1.0 - idle_d / total_d)) as usize
    }

    fn parse(line: &str) -> Self {
        let parts = line.split(" ").collect::<Vec<_>>();
        let id = parts[0].strip_prefix("cpu").unwrap().parse().unwrap();
        let times = parts[1..]
            .iter()
            .map(|n| n.parse::<usize>().unwrap())
            .collect::<Vec<_>>();
        let idle = times[3];
        let total = times.iter().sum();
        Self { id, idle, total }
    }
}

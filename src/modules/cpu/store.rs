pub(crate) struct Store(Option<Vec<(u8, u64, u64)>>);

impl Store {
    pub(crate) fn new() -> Self {
        Self(None)
    }

    pub(crate) fn update(&mut self, next: Vec<(u8, u64, u64)>) -> Vec<u8> {
        let usage_per_core = if let Some(previous) = self.0.take() {
            assert!(
                previous.len() == next.len(),
                "number of CPU cores has changed from {} to {} (bug?)",
                previous.len(),
                next.len()
            );

            fn load_comparing_to(
                (next_id, next_idle, next_total): (u8, u64, u64),
                (prev_id, prev_idle, prev_total): (u8, u64, u64),
            ) -> u8 {
                assert!(
                    next_id == prev_id,
                    "CPU id mismatch: {next_id} vs {prev_id}",
                );

                let idle_d = (next_idle - prev_idle) as f64;
                let total_d = (next_total - prev_total) as f64;

                (100.0 * (1.0 - idle_d / total_d)) as u8
            }

            previous
                .iter()
                .zip(next.iter())
                .map(|(prev, next)| load_comparing_to(*next, *prev))
                .collect::<Vec<_>>()
        } else {
            vec![0; next.len()]
        };

        self.0 = Some(next);
        usage_per_core
    }
}

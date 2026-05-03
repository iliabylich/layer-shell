use anyhow::{Context as _, Result, bail};
pub(crate) struct Store(Option<Vec<(u8, u64, u64)>>);

impl Store {
    pub(crate) const fn new() -> Self {
        Self(None)
    }

    pub(crate) fn update(&mut self, next: Vec<(u8, u64, u64)>) -> Result<Vec<u8>> {
        let usage_per_core = if let Some(previous) = self.0.take() {
            if previous.len() != next.len() {
                bail!(
                    "number of CPU cores has changed from {} to {} (bug?)",
                    previous.len(),
                    next.len()
                );
            }

            previous
                .iter()
                .zip(next.iter())
                .map(|(prev, next)| load_comparing_to(*next, *prev))
                .collect::<Result<Vec<_>>>()?
        } else {
            vec![0; next.len()]
        };

        self.0 = Some(next);
        Ok(usage_per_core)
    }
}

fn load_comparing_to(
    (next_id, next_idle, next_total): (u8, u64, u64),
    (prev_id, prev_idle, prev_total): (u8, u64, u64),
) -> Result<u8> {
    if next_id != prev_id {
        bail!("CPU id mismatch: {next_id} vs {prev_id}");
    }

    let idle_d = f64::from(u32::try_from(next_idle - prev_idle).context("values are too large")?);
    let total_d =
        f64::from(u32::try_from(next_total - prev_total).context("values are too large")?);
    let percent = 100.0 * (1.0 - idle_d / total_d);

    let percent = percent as i64;

    u8::try_from(percent).context("percent is too big for u8")
}

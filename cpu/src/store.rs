use crate::core::Core;
use anyhow::{Result, ensure};
use tokio::fs::File;

pub(crate) struct Store {
    state: Option<Vec<Core>>,
}

impl Store {
    pub(crate) fn new() -> Self {
        Self { state: None }
    }

    pub(crate) async fn update(&mut self, f: &mut File, buf: &mut [u8]) -> Result<Vec<u8>> {
        let previous = self.state.take();
        let next = Core::read_and_parse_all(f, buf).await?;

        let usage = if let Some(previous) = previous {
            ensure!(
                previous.len() == next.len(),
                "number of CPU cores has changed from {} to {} (bug?)",
                previous.len(),
                next.len()
            );

            previous
                .iter()
                .zip(next.iter())
                .map(|(prev, next)| load_comparing_to(next, prev))
                .collect::<Result<Vec<_>>>()?
        } else {
            vec![0; next.len()]
        };

        self.state = Some(next);
        Ok(usage)
    }
}

fn load_comparing_to(next: &Core, prev: &Core) -> Result<u8> {
    ensure!(
        next.id == prev.id,
        "CPU id mismatch: {} vs {}",
        next.id,
        prev.id
    );

    let idle_d = (next.idle - prev.idle) as f64;
    let total_d = (next.total - prev.total) as f64;

    Ok((100.0 * (1.0 - idle_d / total_d)) as u8)
}

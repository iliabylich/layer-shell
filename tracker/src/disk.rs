use crate::state::State;
use anyhow::{Context as _, Result};
use std::path::PathBuf;

pub(crate) struct Disk {
    dir: PathBuf,
}

impl Disk {
    pub(crate) async fn new() -> Result<Self> {
        let dir = std::env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .context("no $XDG_STATE_HOME")
            .or_else(|_| {
                let home = std::env::var("HOME").context("no $HOME")?;
                Ok::<_, anyhow::Error>(PathBuf::from(home).join(".local/state"))
            })?
            .join("layer-shell");

        tokio::fs::create_dir_all(&dir)
            .await
            .context("failed to create state dir")?;

        Ok(Self { dir })
    }

    async fn read_at(path: PathBuf) -> Result<State> {
        let text = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("failed to read {path:?}"))?;

        toml::from_str(&text).with_context(|| format!("failed to parse state at {path:?}"))
    }

    async fn max_filename(&self) -> Result<Option<PathBuf>> {
        let mut dir = tokio::fs::read_dir(&self.dir)
            .await
            .context("failed to read_dir")?;

        let mut files = vec![];

        while let Ok(Some(entry)) = dir.next_entry().await {
            let metadata = match entry.metadata().await {
                Ok(metadata) => metadata,
                Err(err) => {
                    log::error!("{err:?}");
                    continue;
                }
            };

            let path = entry.path();

            if !metadata.is_file() {
                log::warn!("unexpected directory in state dir: {path:?}");
                continue;
            }

            files.push(path);
        }

        files.sort_unstable();

        Ok(files.into_iter().next_back())
    }

    pub(crate) async fn read_latest_state(&self) -> Result<State> {
        let Some(path) = self.max_filename().await? else {
            return Ok(State::empty());
        };

        Self::read_at(path).await
    }

    pub(crate) async fn write(&self, state: &State) -> Result<()> {
        let text = toml::to_string_pretty(state).context("failed to serialize state")?;
        let path = self.dir.join(state.created_at().to_string());
        tokio::fs::write(&path, text)
            .await
            .with_context(|| format!("failed to write to {path:?}"))
    }
}

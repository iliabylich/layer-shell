use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy)]
pub enum Command {
    GoToWorkspace(usize),
}

impl Command {
    pub async fn dispatch(self) {
        if let Err(err) = self.try_dispatch().await {
            log::error!("failed to dispatch {self:?}: {:?}", err);
        }
    }

    async fn try_dispatch(self) -> Result<()> {
        match self {
            Command::GoToWorkspace(idx) => go_to_workspace(idx).await,
        }
    }
}

async fn go_to_workspace(idx: usize) -> Result<()> {
    let mut child = tokio::process::Command::new("hyprctl")
        .args(["dispatch", "workspace", &format!("{}", idx + 1)])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("failed to spawn hyprctl")?;

    child.wait().await.context("Failed to spawn hyprctl")?;

    Ok(())
}

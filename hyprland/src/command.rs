use anyhow::{Context, Result};

#[derive(Debug)]
pub struct HyprlandGoToWorkspace {
    pub idx: usize,
}

impl HyprlandGoToWorkspace {
    pub async fn exec(self) {
        if let Err(err) = go_to_workspace(self.idx).await {
            log::error!("failed to dispatch {self:?}: {:?}", err);
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

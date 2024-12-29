use anyhow::{Context, Result};

pub(crate) async fn go_to_workspace(idx: usize) {
    async fn try_exec(idx: usize) -> Result<()> {
        let mut child = tokio::process::Command::new("hyprctl")
            .args(["dispatch", "workspace", &format!("{}", idx + 1)])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("failed to spawn hyprctl")?;

        child.wait().await.context("Failed to spawn hyprctl")?;

        Ok(())
    }

    if let Err(err) = try_exec(idx).await {
        log::error!("failed to dispatch {idx}: {:?}", err);
    }
}

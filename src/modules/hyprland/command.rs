use anyhow::{Context, Result};

pub(crate) fn go_to_workspace(idx: usize) {
    fn try_exec(idx: usize) -> Result<()> {
        let mut child = std::process::Command::new("hyprctl")
            .args(["dispatch", "workspace", &format!("{}", idx + 1)])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("failed to spawn hyprctl")?;

        child.wait().context("Failed to spawn hyprctl")?;

        Ok(())
    }

    if let Err(err) = try_exec(idx) {
        log::error!("failed to dispatch {idx}: {:?}", err);
    }
}

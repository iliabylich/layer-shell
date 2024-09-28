use crate::models::app_list;
use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
pub(crate) enum Command {
    GoToWorkspace { idx: usize },

    LauncherReset,
    LauncherGoUp,
    LauncherGoDown,
    LauncherSetSearch(String),
    LauncherExecSelected,
}

pub(crate) async fn start_processing(mut rx: Receiver<Command>) {
    while let Some(command) = rx.recv().await {
        command.execute().await;
    }
}

impl Command {
    async fn execute(&self) {
        log::info!("Running command {:?}", self);
        use Command::*;

        match self {
            GoToWorkspace { idx } => {
                tokio::process::Command::new("hyprctl")
                    .args(["dispatch", "workspace", &format!("{}", idx + 1)])
                    .spawn()
                    .unwrap()
                    .wait()
                    .await
                    .unwrap();
            }

            LauncherReset | LauncherGoUp | LauncherGoDown | LauncherSetSearch(_)
            | LauncherExecSelected => app_list::on_command(self).await,
        }
    }
}

use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
pub(crate) enum Command {
    GoToWorkspace { idx: usize },
}

pub(crate) async fn start_processing(mut rx: Receiver<Command>) {
    while let Some(command) = rx.recv().await {
        command.execute().await;
    }
}

impl Command {
    async fn execute(&self) {
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
        }
    }
}

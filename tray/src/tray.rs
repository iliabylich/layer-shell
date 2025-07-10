use crate::{TrayEvent, tray_task::TrayTask};
use futures::Stream;
use pin_project_lite::pin_project;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

pin_project! {
    pub struct Tray {
        #[pin]
        erx: UnboundedReceiver<TrayEvent>,
    }
}

const NAME: &str = "Tray";

impl Tray {
    pub fn spawn(token: CancellationToken) -> (&'static str, Self, JoinHandle<()>, TrayCtl) {
        let (etx, erx) = tokio::sync::mpsc::unbounded_channel::<TrayEvent>();
        let (ctx, crx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let handle = tokio::task::spawn(async move {
            if let Err(err) = TrayTask::start(etx, crx, token).await {
                log::error!(target: "Tray", "{err:?}");
            }
        });
        let trigger = TrayCtl { tx: ctx };
        (NAME, Self { erx }, handle, trigger)
    }
}

impl Stream for Tray {
    type Item = TrayEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().erx.poll_recv(cx)
    }
}

pub struct TrayCtl {
    tx: UnboundedSender<String>,
}

impl TrayCtl {
    pub fn trigger(&self, uuid: String) {
        if let Err(err) = self.tx.send(uuid) {
            log::error!(target: "Tray", "failed to trigger Tray; channel is closed ({err:?})")
        }
    }
}

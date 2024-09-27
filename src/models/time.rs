use crate::models::Event;
use chrono::Local;
use tokio::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    loop {
        let now = Local::now();
        tx.send(Event::Time {
            time: now.format("%H:%M:%S").to_string(),
            date: now.format("%Y %B %e").to_string(),
        })
        .await
        .unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

use crate::utils::singleton;
use chrono::Local;

pub(crate) struct Time {
    callbacks: Vec<fn(TimeData)>,
}
singleton!(Time);

impl Time {
    pub(crate) async fn spawn() {
        Self::set(Self { callbacks: vec![] });

        loop {
            let now = Local::now();
            let data = TimeData {
                label: now.format("%H:%M:%S").to_string(),
                tooltip: now.format("%Y %B %e\n%A").to_string(),
            };
            for callback in this().callbacks.iter() {
                (callback)(data.clone());
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    pub(crate) fn subscribe(f: fn(TimeData)) {
        this().callbacks.push(f);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TimeData {
    pub(crate) label: String,
    pub(crate) tooltip: String,
}

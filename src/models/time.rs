use chrono::{DateTime, Local};

pub(crate) struct Time;

impl Time {
    pub(crate) fn spawn<F>(f: F)
    where
        F: Fn(DateTime<Local>) + 'static,
    {
        gtk4::glib::spawn_future_local(async move {
            loop {
                let datetime = Local::now();
                f(datetime);
                gtk4::glib::timeout_future_seconds(1).await;
            }
        });
    }
}

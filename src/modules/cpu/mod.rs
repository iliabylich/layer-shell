use async_stream::stream;
use futures::Stream;

mod cpu_core_info;
use crate::Event;
use cpu_core_info::CpuCoreInfo;

pub(crate) fn connect() -> impl Stream<Item = Event> {
    let mut previous: Option<Vec<CpuCoreInfo>> = None;

    stream! {
        loop {
            match CpuCoreInfo::parse_current_comparing_to(&mut previous).await {
                Ok(usage_per_core) => {
                    let usage_per_core = Event::CpuUsage { usage_per_core: usage_per_core.into() };
                    yield usage_per_core;
                },
                Err(err) => log::error!("failed to retrieve CPU usage: {:?}", err)
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}

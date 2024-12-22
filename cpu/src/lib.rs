use async_stream::stream;
use futures::Stream;

mod cpu_core_info;
use cpu_core_info::CpuCoreInfo;

#[derive(Debug)]
pub struct CpuUsage(pub Vec<usize>);

pub fn connect() -> impl Stream<Item = CpuUsage> {
    let mut previous: Option<Vec<CpuCoreInfo>> = None;

    stream! {
        loop {
            match CpuCoreInfo::parse_current_comparing_to(&mut previous).await {
                Ok(usage_per_core) => yield CpuUsage(usage_per_core),
                Err(err) => log::error!("failed to retrieve CPU usage: {:?}", err)
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}

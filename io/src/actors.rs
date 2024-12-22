use crate::Event;
use futures::{pin_mut, Stream};
use layer_shell_hyprland::HyprlandEvent;
use layer_shell_weather::WeatherEvent;
use std::sync::mpsc::Sender;
use tokio_stream::StreamExt as _;

pub(crate) async fn spawn_all(tx: Sender<Event>) {
    let stream = merged_stream().await;
    pin_mut!(stream);

    while let Some(event) = stream.next().await {
        if let Err(err) = tx.send(event) {
            log::error!("Failed to send event: {:?}", err);
        }
    }
}

async fn merged_stream() -> impl Stream<Item = Event> {
    futures::stream::empty()
        .merge(layer_shell_app_list::connect().await.map(Event::AppList))
        .merge(layer_shell_cpu::connect().map(Event::CpuUsage))
        .merge(
            layer_shell_hyprland::connect()
                .await
                .map(|event| match event {
                    HyprlandEvent::Workspaces(workspaces) => Event::Workspaces(workspaces),
                    HyprlandEvent::Language(lang) => Event::Language(lang),
                }),
        )
        .merge(layer_shell_memory::connect().map(Event::Memory))
        .merge(layer_shell_pipewire::connect().map(Event::Volume))
        .merge(layer_shell_time::connect().map(Event::Time))
        .merge(layer_shell_weather::connect().map(|event| match event {
            WeatherEvent::CurrentWeather(current) => Event::CurrentWeather(current),
            WeatherEvent::ForecastWeather(forecast) => Event::ForecastWeather(forecast),
        }))
}

use crate::{global, Command, Event};
use alsa::{
    mixer::{Selem, SelemChannelId},
    Ctl,
};
use anyhow::{Context, Result};
use std::sync::mpsc::Sender;

global!(MIXER, alsa::Mixer);
global!(OUTPUT_SELEM, Selem<'static>);

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("{}", err);
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    let mixer = alsa::Mixer::new("default", true).context("failed to connect to alsa")?;
    MIXER::set(mixer);

    OUTPUT_SELEM::set(output_device().context("failed to get default output device")?);

    let volume = get_volume()?;
    tx.send(Event::Volume(volume))
        .context("failed to send Volume event")?;

    let ctl = Ctl::new("hw:0", true).context("failed to get ctl")?;
    ctl.subscribe_events(true)
        .context("failed to subscribe to alsa event stream")?;
    ctl.wait(Some(50)).context("failed to wait")?;
    loop {
        if let Ok(Some(event)) = ctl.read() {
            if OUTPUT_SELEM::get().get_id().get_index() == event.get_id().get_index() {
                MIXER::get()
                    .handle_events()
                    .context("failed to receive alsa events")?;

                let volume = get_volume()?;
                tx.send(Event::Volume(volume))
                    .context("failed to send Volume event")?;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

fn output_device() -> Option<Selem<'static>> {
    for elem in MIXER::get().iter() {
        if let Some(selem) = Selem::new(elem) {
            if selem.can_playback() {
                return Some(selem);
            }
        }
    }

    None
}

fn get_volume() -> Result<f64> {
    MIXER::get()
        .handle_events()
        .context("failed to receive alsa events")?;

    if let Ok(volume) = OUTPUT_SELEM::get().get_playback_volume(SelemChannelId::mono()) {
        let (_, max) = OUTPUT_SELEM::get().get_playback_volume_range();
        let volume = (volume as f64) / (max as f64);
        Ok(volume)
    } else {
        anyhow::bail!("failed to get volume from alsa")
    }
}

fn set_volume(volume: f64) -> Result<()> {
    let (_, max) = OUTPUT_SELEM::get().get_playback_volume_range();
    let volume = (volume * (max as f64)) as i64;
    OUTPUT_SELEM::get()
        .set_playback_volume_all(volume)
        .context("failed to set alsa volume")?;
    Ok(())
}

pub(crate) async fn on_command(command: &Command) {
    if let Command::SetVolume(volume) = command {
        if let Err(err) = set_volume(*volume) {
            log::error!("failed to set volume: {}", err)
        }
    }
}

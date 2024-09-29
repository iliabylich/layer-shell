use crate::{
    models::{Command, Event},
    utils::global,
};
use alsa::{
    mixer::{Selem, SelemChannelId},
    Ctl,
};
use tokio::sync::mpsc::Sender;

global!(MIXER, alsa::Mixer);
global!(OUTPUT_SELEM, Selem<'static>);

pub(crate) async fn spawn(tx: Sender<Event>) {
    let mixer = alsa::Mixer::new("default", true).unwrap();
    MIXER::set(mixer);

    if let Some(selem) = MIXER::get()
        .iter()
        .map(|elem| Selem::new(elem).unwrap())
        .find(|selem| selem.can_playback())
    {
        OUTPUT_SELEM::set(selem);
    } else {
        log::error!("Failed to get default output device");
        return;
    }

    if let Some(volume) = get_volume() {
        if tx.send(Event::Volume(volume)).await.is_err() {
            log::error!("failed to send Volume event");
        }
    }

    let ctl = Ctl::new("hw:0", true).unwrap();
    ctl.subscribe_events(true).unwrap();
    ctl.wait(Some(50)).unwrap();
    loop {
        if let Ok(Some(event)) = ctl.read() {
            if OUTPUT_SELEM::get().get_id().get_index() == event.get_id().get_index() {
                MIXER::get().handle_events().unwrap();
                if let Some(volume) = get_volume() {
                    if tx.send(Event::Volume(volume)).await.is_err() {
                        log::error!("failed to send Volume event");
                    }
                } else {
                    log::error!("failed to get volume");
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

fn get_volume() -> Option<f64> {
    MIXER::get().handle_events().unwrap();
    if let Ok(volume) = OUTPUT_SELEM::get().get_playback_volume(SelemChannelId::mono()) {
        let (_, max) = OUTPUT_SELEM::get().get_playback_volume_range();
        let volume = (volume as f64) / (max as f64);
        Some(volume)
    } else {
        None
    }
}

fn set_volume(volume: f64) {
    let (_, max) = OUTPUT_SELEM::get().get_playback_volume_range();
    let volume = (volume * (max as f64)) as i64;
    OUTPUT_SELEM::get().set_playback_volume_all(volume).unwrap();
}

pub(crate) async fn on_command(command: &Command) {
    if let Command::SetVolume(volume) = command {
        set_volume(*volume);
    }
}

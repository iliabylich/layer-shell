use tokio::sync::mpsc::Sender;

use super::fire_event_on_current_thread;
use crate::{
    ffi::gvc,
    models::{Command, Event},
    utils::singleton,
};

pub(crate) async fn spawn(_tx: Sender<Event>) {
    let control = gvc::MixerControl::new();

    OutputSound::set(OutputSound {
        control,
        subscription: None,
    });

    control.connect_default_sink_changed(on_output_changed);
    control.open();
}

struct OutputSound {
    control: gvc::MixerControl,
    subscription: Option<Subscription>,
}
singleton!(OutputSound);

struct Subscription {
    stream: gvc::MixerStream,
    sub_id: u64,
}

unsafe extern "C" fn on_output_changed(control: gvc::MixerControl, id: std::ffi::c_uint) {
    if let Some(Subscription { stream, sub_id }) = OutputSound::get().subscription {
        stream.disconnect(sub_id);
    }
    if let Some(stream) = control.lookup_stream_id(id) {
        let sub_id = stream.connect_volume_changed(on_volume_changed);
        OutputSound::get().subscription = Some(Subscription { stream, sub_id });

        on_volume_changed();
    }
}

unsafe extern "C" fn on_volume_changed() {
    // GVC is based on glib which calls all GObject callbacks
    // on the main UI thread. Yes, here we are in UI thread,
    // so it's safe to directly fire an event here.
    fire_event_on_current_thread(&Event::Volume(current_volume()));
}

unsafe fn current_volume() -> f64 {
    let control = OutputSound::get().control;
    if let Some(Subscription { stream, .. }) = OutputSound::get().subscription {
        let max = control.get_vol_max_norm();
        let volume = stream.get_volume() as f64;
        return volume / max;
    }
    0.0
}

pub(crate) async fn on_command(command: &Command) {
    if let Command::SetVolume(volume) = command {
        let control = OutputSound::get().control;
        let max = control.get_vol_max_norm();
        if let Some(Subscription { stream, .. }) = OutputSound::get().subscription {
            stream.set_volume((volume * max) as u32);
            stream.push_volume();
        }
    }
}

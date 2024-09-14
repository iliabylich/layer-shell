use crate::{
    ffi::gvc::{self, MixerControl},
    models::singleton,
};

pub(crate) struct OutputSound {
    on_change: Box<dyn Fn(f64)>,
    control: gvc::MixerControl,
}
singleton!(OutputSound, OUTPUT_SOUND_INSTANCE);

struct OutputSubscription {
    stream: gvc::MixerStream,
    sub_id: u64,
}
static mut OUTPUT_SUBSCRIPTION_INSTANCE: Option<OutputSubscription> = None;

unsafe extern "C" fn on_output_changed(control: MixerControl, id: std::ffi::c_uint) {
    if let Some(OutputSubscription { stream, sub_id }) = OUTPUT_SUBSCRIPTION_INSTANCE {
        stream.disconnect(sub_id);
    }
    if let Some(stream) = control.lookup_stream_id(id) {
        let sub_id = stream.connect_volume_changed(on_volume_changed);
        OUTPUT_SUBSCRIPTION_INSTANCE = Some(OutputSubscription { stream, sub_id });

        on_volume_changed();
    }
}

unsafe extern "C" fn on_volume_changed() {
    let f = &OutputSound::get().on_change;
    f(current_volume())
}

unsafe fn current_volume() -> f64 {
    let control = OutputSound::get().control;
    if let Some(OutputSubscription { stream, .. }) = OUTPUT_SUBSCRIPTION_INSTANCE {
        let max = control.get_vol_max_norm();
        let volume = stream.get_volume() as f64;
        return volume / max;
    }
    0.0
}

impl OutputSound {
    pub(crate) fn spawn<F>(f: F)
    where
        F: Fn(f64) + 'static,
    {
        let control = MixerControl::new();

        Self::set(Self {
            control,
            on_change: Box::new(f),
        });

        control.connect_default_sink_changed(on_output_changed);

        control.open();
    }

    pub(crate) fn set_volume(value: f64) {
        unsafe {
            let control = OutputSound::get().control;
            let max = control.get_vol_max_norm();
            if let Some(OutputSubscription { stream, .. }) = OUTPUT_SUBSCRIPTION_INSTANCE {
                stream.set_volume((value * max) as u32);
                stream.push_volume();
            }
        }
    }
}

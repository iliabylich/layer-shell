use crate::{ffi::gvc, utils::singleton};

pub(crate) struct OutputSound {
    handlers: Vec<Box<dyn Fn(f64)>>,
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
    for f in OutputSound::get().handlers.iter() {
        f(current_volume())
    }
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

impl OutputSound {
    pub(crate) fn spawn() {
        let control = gvc::MixerControl::new();

        Self::set(Self {
            control,
            handlers: vec![],
            subscription: None,
        });

        control.connect_default_sink_changed(on_output_changed);
        control.open();
    }

    pub(crate) fn subscribe<F>(f: F)
    where
        F: Fn(f64) + 'static,
    {
        Self::get().handlers.push(Box::new(f));
    }

    pub(crate) fn set_volume(value: f64) {
        let control = OutputSound::get().control;
        let max = control.get_vol_max_norm();
        if let Some(Subscription { stream, .. }) = OutputSound::get().subscription {
            stream.set_volume((value * max) as u32);
            stream.push_volume();
        }
    }
}

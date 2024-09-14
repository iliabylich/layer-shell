use std::os::raw::c_void;

use gtk4::glib::{
    ffi::gboolean,
    gobject_ffi::{g_signal_connect_data, g_signal_handler_disconnect, G_CONNECT_DEFAULT},
};

use super::singleton;

#[repr(C)]
struct GvcMixerControl {
    opaque: u8,
}

#[repr(C)]
struct GvcMixerStream {
    opaque: u8,
}

extern "C" {
    fn gvc_mixer_control_new(name: *const u8) -> *mut GvcMixerControl;
    fn gvc_mixer_control_open(control: *mut GvcMixerControl) -> gboolean;
    fn gvc_mixer_control_lookup_stream_id(
        control: *mut GvcMixerControl,
        id: std::ffi::c_uint,
    ) -> *mut GvcMixerStream;
    fn gvc_mixer_control_get_vol_max_norm(control: *mut GvcMixerControl) -> std::ffi::c_double;
    fn gvc_mixer_stream_get_volume(stream: *mut GvcMixerStream) -> u32;
    fn gvc_mixer_stream_set_volume(stream: *mut GvcMixerStream, volume: u32);
    fn gvc_mixer_stream_push_volume(stream: *mut GvcMixerStream);
}

pub(crate) struct OutputSound {
    on_change: Box<dyn Fn(f64)>,
    control: *mut GvcMixerControl,
}
singleton!(OutputSound, OUTPUT_SOUND_INSTANCE);

struct OutputSubscription {
    stream: *mut GvcMixerStream,
    sub_id: u64,
}
static mut OUTPUT_SUBSCRIPTION_INSTANCE: Option<OutputSubscription> = None;

unsafe extern "C" fn on_output_changed(control: *mut GvcMixerControl, id: std::ffi::c_uint) {
    if let Some(OutputSubscription { stream, sub_id }) = OUTPUT_SUBSCRIPTION_INSTANCE {
        g_signal_handler_disconnect(stream.cast(), sub_id)
    }
    let stream = gvc_mixer_control_lookup_stream_id(control, id);
    if stream.is_null() {
        return;
    }
    let sub_id = g_signal_connect_data(
        stream.cast(),
        "notify::volume\0".as_ptr().cast(),
        Some(std::mem::transmute(on_volume_changed as *mut c_void)),
        std::ptr::null_mut(),
        None,
        G_CONNECT_DEFAULT,
    );
    OUTPUT_SUBSCRIPTION_INSTANCE = Some(OutputSubscription { stream, sub_id });

    on_volume_changed();
}

unsafe extern "C" fn on_volume_changed() {
    let f = &OutputSound::get().on_change;
    f(current_volume())
}

unsafe fn current_volume() -> f64 {
    let control = OutputSound::get().control;
    if let Some(OutputSubscription { stream, .. }) = OUTPUT_SUBSCRIPTION_INSTANCE {
        let max = gvc_mixer_control_get_vol_max_norm(control);
        let volume = gvc_mixer_stream_get_volume(stream) as f64;
        return volume / max;
    }
    0.0
}

impl OutputSound {
    pub(crate) fn spawn<F>(f: F)
    where
        F: Fn(f64) + 'static,
    {
        unsafe {
            let control = gvc_mixer_control_new("layer-shell-mixer-control".as_ptr());

            Self::set(Self {
                control,
                on_change: Box::new(f),
            });

            g_signal_connect_data(
                control.cast(),
                "default-sink-changed\0".as_ptr().cast(),
                Some(std::mem::transmute(on_output_changed as *mut c_void)),
                std::ptr::null_mut(),
                None,
                G_CONNECT_DEFAULT,
            );

            gvc_mixer_control_open(control);
        }
    }

    pub(crate) fn set_volume(value: f64) {
        unsafe {
            let control = OutputSound::get().control;
            let max = gvc_mixer_control_get_vol_max_norm(control);
            if let Some(OutputSubscription { stream, .. }) = OUTPUT_SUBSCRIPTION_INSTANCE {
                gvc_mixer_stream_set_volume(stream, (value * max) as u32);
                gvc_mixer_stream_push_volume(stream);
            }
        }
    }
}

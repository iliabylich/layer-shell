use std::ffi::{c_double, c_uint, c_void};

use gtk4::glib::gobject_ffi::{
    g_signal_connect_data, g_signal_handler_disconnect, G_CONNECT_DEFAULT,
};

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
    fn gvc_mixer_control_open(control: *mut GvcMixerControl);
    fn gvc_mixer_control_lookup_stream_id(
        control: *mut GvcMixerControl,
        id: c_uint,
    ) -> *mut GvcMixerStream;
    fn gvc_mixer_control_get_vol_max_norm(control: *mut GvcMixerControl) -> c_double;

    fn gvc_mixer_stream_get_volume(stream: *mut GvcMixerStream) -> u32;
    fn gvc_mixer_stream_set_volume(stream: *mut GvcMixerStream, volume: u32);
    fn gvc_mixer_stream_push_volume(stream: *mut GvcMixerStream);
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct MixerControl {
    raw: *mut GvcMixerControl,
}

impl MixerControl {
    pub(crate) fn new() -> Self {
        let control = unsafe { gvc_mixer_control_new("layer-shell-mixer-control".as_ptr()) };

        Self { raw: control }
    }

    pub(crate) fn open(&self) {
        unsafe { gvc_mixer_control_open(self.raw) }
    }

    pub(crate) fn lookup_stream_id(&self, id: c_uint) -> Option<MixerStream> {
        let stream = unsafe { gvc_mixer_control_lookup_stream_id(self.raw, id) };
        if stream.is_null() {
            None
        } else {
            Some(MixerStream { raw: stream })
        }
    }

    pub(crate) fn get_vol_max_norm(&self) -> c_double {
        unsafe { gvc_mixer_control_get_vol_max_norm(self.raw) }
    }

    pub(crate) fn connect_default_sink_changed(&self, f: unsafe extern "C" fn(Self, c_uint)) {
        unsafe {
            g_signal_connect_data(
                self.raw.cast(),
                "default-sink-changed\0".as_ptr().cast(),
                Some(std::mem::transmute(f as *mut c_void)),
                std::ptr::null_mut(),
                None,
                G_CONNECT_DEFAULT,
            );
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct MixerStream {
    raw: *mut GvcMixerStream,
}

impl MixerStream {
    pub(crate) fn get_volume(&self) -> u32 {
        unsafe { gvc_mixer_stream_get_volume(self.raw) }
    }
    pub(crate) fn set_volume(&self, volume: u32) {
        unsafe {
            gvc_mixer_stream_set_volume(self.raw, volume);
        }
    }
    pub(crate) fn push_volume(&self) {
        unsafe {
            gvc_mixer_stream_push_volume(self.raw);
        }
    }
    pub(crate) fn connect_volume_changed(&self, f: unsafe extern "C" fn()) -> u64 {
        unsafe {
            g_signal_connect_data(
                self.raw.cast(),
                "notify::volume\0".as_ptr().cast(),
                Some(std::mem::transmute(f as *mut c_void)),
                std::ptr::null_mut(),
                None,
                G_CONNECT_DEFAULT,
            )
        }
    }
    pub(crate) fn disconnect(&self, sub_id: u64) {
        unsafe {
            g_signal_handler_disconnect(self.raw.cast(), sub_id);
        }
    }
}

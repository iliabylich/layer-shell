use gtk4::{
    prelude::{AdjustmentExt, RangeExt},
    Image, Scale,
};

use crate::{globals::load_widget, models::OutputSound};

pub(crate) struct Sound;

impl Sound {
    pub(crate) fn init() {
        let icon = load_widget::<Image>("SoundImage");
        let scale = load_widget::<Scale>("SoundScale");

        OutputSound::spawn(|volume| {
            scale.set_value(volume);
            if volume == 0.0 {
                icon.set_icon_name(Some("audio-volume-muted-symbolic"));
            } else if volume >= 0.01 && volume < 0.34 {
                icon.set_icon_name(Some("audio-volume-low-symbolic"));
            } else if volume >= 0.34 && volume < 0.67 {
                icon.set_icon_name(Some("audio-volume-medium-symbolic"));
            } else if volume >= 0.67 && volume < 1.0 {
                icon.set_icon_name(Some("audio-volume-high-symbolic"));
            } else {
                icon.set_icon_name(Some("audio-volume-overamplified-symbolic"));
            }
        });

        scale.connect_change_value(|_, _, _| {
            let mut volume = scale.adjustment().value();
            if volume > 1.0 {
                volume = 1.0
            }
            OutputSound::set_volume(volume);
            gtk4::glib::Propagation::Proceed
        });
    }
}

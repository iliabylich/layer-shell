use crate::{globals::load_widget, models::OutputSound};
use gtk4::{
    prelude::{AdjustmentExt, RangeExt},
    Image, Scale,
};

pub(crate) fn init() {
    let icon = load_widget::<Image>("SoundWidgetImage");
    let scale = load_widget::<Scale>("SoundWidgetScale");

    OutputSound::subscribe(|volume| {
        scale.set_value(volume);
        icon.set_icon_name(Some(volume_to_icon(volume)));
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

fn volume_to_icon(volume: f64) -> &'static str {
    if volume == 0.0 {
        "audio-volume-muted-symbolic"
    } else if (0.01..0.34).contains(&volume) {
        "audio-volume-low-symbolic"
    } else if (0.34..0.67).contains(&volume) {
        "audio-volume-medium-symbolic"
    } else if (0.67..1.0).contains(&volume) {
        "audio-volume-high-symbolic"
    } else {
        "audio-volume-overamplified-symbolic"
    }
}

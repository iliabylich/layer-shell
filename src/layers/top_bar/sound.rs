use gtk4::{
    prelude::{AdjustmentExt, RangeExt},
    Image, Scale,
};

use crate::{globals::load_widget, models::OutputSound, utils::TypedChildren};

pub(crate) fn init() {
    let widget = load_widget::<gtk4::Box>("SoundWidget");
    let icon = widget.first_child_as::<Image>();
    let scale = widget.last_child_as::<Scale>();

    OutputSound::spawn(|volume| {
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

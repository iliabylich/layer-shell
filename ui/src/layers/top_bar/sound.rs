use crate::widgets::{SoundWidgetImage, SoundWidgetScale};
use gtk4::prelude::{AdjustmentExt, RangeExt};
use layer_shell_io::{publish, subscribe, Command, Event};

pub(crate) fn init() {
    subscribe(on_event);

    SoundWidgetScale().connect_change_value(|_, _, _| {
        let mut volume = SoundWidgetScale().adjustment().value();
        if volume > 1.0 {
            volume = 1.0
        }
        publish(Command::SetVolume(volume));
        gtk4::glib::Propagation::Proceed
    });
}

fn on_event(event: &Event) {
    if let Event::Volume(volume) = event {
        SoundWidgetScale().set_value(*volume);
        SoundWidgetImage().set_icon_name(Some(volume_to_icon(*volume)));
    }
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

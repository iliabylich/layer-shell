use crate::widgets::{SoundWidget, SoundWidgetImage, SoundWidgetScale};
use gtk4::{
    prelude::{AdjustmentExt, RangeExt},
    PropagationPhase,
};
use layer_shell_io::{publish, subscribe, Command, Event};
use vte4::{EventControllerExt, WidgetExt};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::Volume(volume) = event {
            SoundWidgetScale().set_value(*volume as f64);
            SoundWidgetImage().set_icon_name(Some(volume_to_icon(*volume)));
        }
    });

    let ctrl = gtk4::GestureClick::new();
    ctrl.set_propagation_phase(PropagationPhase::Capture);
    ctrl.connect_released(|_, _, _, _| {
        let volume = SoundWidgetScale().adjustment().value().clamp(0.0, 1.0);
        publish(Command::SetVolume(volume as f32));
    });
    SoundWidget().add_controller(ctrl);
}

fn volume_to_icon(volume: f32) -> &'static str {
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

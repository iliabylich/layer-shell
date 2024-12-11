use crate::widgets::top_bar::sound::{Image, Scale, Widget};
use gtk4::{
    prelude::{AdjustmentExt, EventControllerExt, RangeExt, WidgetExt},
    PropagationPhase,
};
use layer_shell_io::{publish, subscribe, Command, Event};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::Volume(event) = event {
            Scale().set_value(event.volume as f64);
            Image().set_icon_name(Some(volume_to_icon(event.volume)));
        }
    });

    let ctrl = gtk4::GestureClick::new();
    ctrl.set_propagation_phase(PropagationPhase::Capture);
    ctrl.connect_released(|_, _, _, _| {
        let volume = Scale().adjustment().value().clamp(0.0, 1.0);
        publish(Command::SetVolume(volume as f32));
    });
    Widget().add_controller(ctrl);
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

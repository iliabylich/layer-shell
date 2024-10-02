use crate::globals::load_widget;
use gtk4::{prelude::WidgetExt, Label};
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(on_event);
}

fn on_event(event: &Event) {
    if let Event::Time { time, date } = event {
        let label = load_widget::<Label>("ClockWidgetLabel");
        label.set_label(time);
        label.set_tooltip_text(Some(date));
    }
}

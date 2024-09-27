use crate::{
    globals::load_widget,
    models::{subscribe, Event},
};
use gtk4::{prelude::WidgetExt, Label};

pub(crate) fn init() {
    subscribe(on_change);
}

fn on_change(event: &Event) {
    let Event::Time { time, date } = event else {
        return;
    };
    let label = load_widget::<Label>("ClockWidgetLabel");
    label.set_label(time);
    label.set_tooltip_text(Some(date));
}

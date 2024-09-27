use crate::{
    globals::load_widget,
    models::{Time, TimeData},
};
use gtk4::{prelude::WidgetExt, Label};

pub(crate) fn init() {
    fn on_change(data: TimeData) {
        let label = load_widget::<Label>("ClockWidgetLabel");
        label.set_label(&data.label);
        label.set_tooltip_text(Some(&data.tooltip));
    }

    Time::subscribe(on_change);
}

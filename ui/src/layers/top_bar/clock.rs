use crate::widgets::top_bar::clock::Label;
use gtk4::prelude::WidgetExt;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::Time { time, date } = event {
            Label().set_label(time);
            Label().set_tooltip_text(Some(date));
        }
    });
}

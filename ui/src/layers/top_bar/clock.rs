use crate::widgets::top_bar::clock::Label;
use gtk4::prelude::WidgetExt;
use layer_shell_io::{subscribe, time::Time, Event};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::Time(Time { time, date }) = event {
            Label().set_label(time);
            Label().set_tooltip_text(Some(date));
        }
    });
}

use crate::widgets::top_bar::language::Label;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::Language(event) = event {
            Label().set_label(match event.lang.as_str() {
                "English (US)" => "EN",
                "Polish" => "PL",
                _ => "??",
            });
        }
    });
}

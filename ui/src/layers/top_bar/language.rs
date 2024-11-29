use crate::widgets::top_bar::language::Label;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::Language(lang) = event {
            Label().set_label(match lang.as_str() {
                "English (US)" => "EN",
                "Polish" => "PL",
                _ => "??",
            });
        }
    });
}

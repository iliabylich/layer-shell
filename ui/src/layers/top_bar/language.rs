use crate::widgets::LanguageWidgetLabel;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::Language { lang } = event {
            LanguageWidgetLabel().set_label(match lang.as_str() {
                "English (US)" => "EN",
                "Polish" => "PL",
                _ => "??",
            });
        }
    });
}

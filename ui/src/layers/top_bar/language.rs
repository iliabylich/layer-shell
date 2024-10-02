use crate::globals::load_widget;
use gtk4::Label;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(on_event);
}

fn on_event(event: &Event) {
    if let Event::Language { lang } = event {
        let label = load_widget::<Label>("LanguageWidgetLabel");
        label.set_label(map_language(lang));
    }
}

fn map_language(lang: &str) -> &'static str {
    match lang {
        "English (US)" => "EN",
        "Polish" => "PL",
        _ => "??",
    }
}

use crate::{
    globals::load_widget,
    models::{subscribe, Event},
};
use gtk4::Label;

pub(crate) fn init() {
    subscribe(on_change);
}

fn on_change(event: &Event) {
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

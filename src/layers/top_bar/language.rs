use crate::{
    globals::load_widget,
    models::{subscribe, Event},
};
use gtk4::Label;

pub(crate) fn init() {
    subscribe(on_change);
}

fn on_change(event: &Event) {
    match event {
        Event::Language { lang } => {
            let label = load_widget::<Label>("LanguageWidgetLabel");
            label.set_label(map_language(lang));
        }
        _ => {}
    }
}

fn map_language(lang: &str) -> &'static str {
    match lang {
        "English (US)" => "EN",
        "Polish" => "PL",
        _ => "??",
    }
}

use crate::{globals::load_widget, models::HyprlandLanguage};
use gtk4::Label;

pub(crate) fn init() {
    let label = load_widget::<Label>("LanguageWidgetLabel");

    HyprlandLanguage::subscribe(move |lang| {
        label.set_label(map_language(&lang));
    })
}

fn map_language(lang: &str) -> &'static str {
    match lang {
        "English (US)" => "EN",
        "Polish" => "PL",
        _ => "??",
    }
}

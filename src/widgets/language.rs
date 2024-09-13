use gtk4::Label;

use crate::{models::HyprlandLanguage, utils::load_widget};

pub(crate) struct Language;

fn map_language(lang: &str) -> &'static str {
    match lang {
        "English (US)" => "EN",
        "Polish" => "PL",
        _ => "??",
    }
}

impl Language {
    pub(crate) fn init() {
        let label: Label = load_widget("LanguageLabel");

        HyprlandLanguage::spawn(move |lang| {
            label.set_label(map_language(&lang));
        })
    }
}

use gtk4::{CenterBox, Label};

use crate::{globals::load_widget, models::HyprlandLanguage, utils::TypedChildren};

pub(crate) fn init() {
    let widget = load_widget::<CenterBox>("LanguageWidget");
    let [label] = widget.children_as::<1, Label>();

    HyprlandLanguage::spawn(move |lang| {
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

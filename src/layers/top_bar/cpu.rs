use gtk4::Label;

use crate::{globals::load_widget, models::CPU as CPULoad, utils::TypedChildren};

pub(crate) fn init() {
    let widget = load_widget::<gtk4::Box>("CPUWidget");
    let labels = widget.children_as::<12, Label>();

    CPULoad::spawn(move |usage| {
        assert_eq!(usage.len(), labels.len());

        for (idx, load) in usage.iter().enumerate() {
            labels[idx].set_label(indicator(*load));
        }
    });
}

const INDICATORS: &[&str] = &[
    "<span color='#FFFFFF'>▁</span>",
    "<span color='#FFD5D5'>▂</span>",
    "<span color='#FFAAAA'>▃</span>",
    "<span color='#FF8080'>▄</span>",
    "<span color='#FF5555'>▅</span>",
    "<span color='#FF2B2B'>▆</span>",
    "<span color='#FF0000'>▇</span>",
    "<span color='#E60000'>█</span>",
];

fn indicator(load: usize) -> &'static str {
    let mut idx = (load as f64 / 100.0 * INDICATORS.len() as f64) as usize;
    if idx == INDICATORS.len() {
        idx -= 1;
    }
    INDICATORS[idx]
}

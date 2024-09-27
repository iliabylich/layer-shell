use crate::{globals::load_widget, models::CPU};
use gtk4::Label;

pub(crate) fn init() {
    let labels = [
        load_widget::<Label>("CPUWidgetLabel1"),
        load_widget::<Label>("CPUWidgetLabel2"),
        load_widget::<Label>("CPUWidgetLabel3"),
        load_widget::<Label>("CPUWidgetLabel4"),
        load_widget::<Label>("CPUWidgetLabel5"),
        load_widget::<Label>("CPUWidgetLabel6"),
        load_widget::<Label>("CPUWidgetLabel7"),
        load_widget::<Label>("CPUWidgetLabel8"),
        load_widget::<Label>("CPUWidgetLabel9"),
        load_widget::<Label>("CPUWidgetLabel10"),
        load_widget::<Label>("CPUWidgetLabel11"),
        load_widget::<Label>("CPUWidgetLabel12"),
    ];

    CPU::subscribe(move |usage| {
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

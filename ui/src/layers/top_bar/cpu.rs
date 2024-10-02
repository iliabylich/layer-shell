use crate::globals::load_widget;
use gtk4::Label;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(on_event);
}

fn on_event(event: &Event) {
    if let Event::Cpu { usage_per_core } = event {
        let labels = labels();

        assert_eq!(usage_per_core.len(), labels.len());

        for (idx, load) in usage_per_core.iter().enumerate() {
            labels[idx].set_label(indicator(*load));
        }
    }
}

fn labels() -> [&'static Label; 12] {
    [
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
    ]
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

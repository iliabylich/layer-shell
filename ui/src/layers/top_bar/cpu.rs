use crate::widgets;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::Cpu { usage_per_core } = event {
            assert_eq!(usage_per_core.len(), widgets::cpu::labels().len());

            for (idx, load) in usage_per_core.iter().enumerate() {
                widgets::cpu::labels()[idx].set_label(indicator(*load));
            }
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

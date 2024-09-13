use gtk4::Label;

use crate::{globals::load_widget, models::CPU as CPULoad};

pub(crate) struct CPU;

static mut LABELS: Option<Vec<&'static Label>> = None;
fn labels() -> &'static [&'static Label] {
    unsafe { LABELS.as_ref().unwrap() }
}

impl CPU {
    pub(crate) fn init() {
        unsafe {
            LABELS = Some(vec![
                load_widget("CPU1"),
                load_widget("CPU2"),
                load_widget("CPU3"),
                load_widget("CPU4"),
                load_widget("CPU5"),
                load_widget("CPU6"),
                load_widget("CPU7"),
                load_widget("CPU8"),
                load_widget("CPU9"),
                load_widget("CPU10"),
                load_widget("CPU11"),
                load_widget("CPU12"),
            ]);
        }

        CPULoad::spawn(move |usage| {
            assert_eq!(usage.len(), labels().len());

            for (idx, load) in usage.iter().enumerate() {
                labels()[idx].set_label(indicator(*load));
            }
        });
    }
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

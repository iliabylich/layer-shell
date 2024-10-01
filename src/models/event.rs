use std::collections::HashSet;

#[derive(Debug, Clone)]
pub(crate) enum Event {
    Memory {
        used: f64,
        total: f64,
    },
    Cpu {
        usage_per_core: Vec<usize>,
    },
    Time {
        time: String,
        date: String,
    },
    Workspaces {
        ids: HashSet<usize>,
        active_id: usize,
    },
    Language {
        lang: String,
    },
    AppList(Vec<App>),
    Volume(f64),
    SessionScreen(usize),
    WeatherCurrent(String),
    WeatherForecast {
        hourly: Vec<(String, String)>,
        daily: Vec<(String, String)>,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct App {
    pub(crate) name: String,
    pub(crate) selected: bool,
    pub(crate) icon: AppIcon,
}
#[derive(Debug, Clone)]
pub(crate) enum AppIcon {
    IconPath(String),
    IconName(String),
}

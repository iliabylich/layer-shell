#[allow(non_snake_case)]
mod gen;
pub(crate) use gen::*;

pub(crate) fn load() {
    const UI: &str = include_str!("../../Widgets.ui");
    let builder = gtk4::Builder::from_string(UI);

    unsafe { init_widgets(&builder) }
}

pub(crate) mod launcher {
    pub(crate) fn rows() -> [&'static gtk4::Box; 5] {
        [
            super::LauncherRow1(),
            super::LauncherRow2(),
            super::LauncherRow3(),
            super::LauncherRow4(),
            super::LauncherRow5(),
        ]
    }

    pub(crate) fn images() -> [&'static gtk4::Image; 5] {
        [
            super::LauncherRow1Image(),
            super::LauncherRow2Image(),
            super::LauncherRow3Image(),
            super::LauncherRow4Image(),
            super::LauncherRow5Image(),
        ]
    }

    pub(crate) fn labels() -> [&'static gtk4::Label; 5] {
        [
            super::LauncherRow1Label(),
            super::LauncherRow2Label(),
            super::LauncherRow3Label(),
            super::LauncherRow4Label(),
            super::LauncherRow5Label(),
        ]
    }
}

pub(crate) mod networks {
    pub(crate) fn rows() -> [&'static gtk4::CenterBox; 5] {
        [
            super::NetworkRow1(),
            super::NetworkRow2(),
            super::NetworkRow3(),
            super::NetworkRow4(),
            super::NetworkRow5(),
        ]
    }
}

pub(crate) mod weather {
    pub(crate) fn hourly_labels() -> [&'static gtk4::Label; 10] {
        [
            super::HourlyForecastLabel1(),
            super::HourlyForecastLabel2(),
            super::HourlyForecastLabel3(),
            super::HourlyForecastLabel4(),
            super::HourlyForecastLabel5(),
            super::HourlyForecastLabel6(),
            super::HourlyForecastLabel7(),
            super::HourlyForecastLabel8(),
            super::HourlyForecastLabel9(),
            super::HourlyForecastLabel10(),
        ]
    }

    pub(crate) fn hourly_images() -> [&'static gtk4::Image; 10] {
        [
            super::HourlyForecastImage1(),
            super::HourlyForecastImage2(),
            super::HourlyForecastImage3(),
            super::HourlyForecastImage4(),
            super::HourlyForecastImage5(),
            super::HourlyForecastImage6(),
            super::HourlyForecastImage7(),
            super::HourlyForecastImage8(),
            super::HourlyForecastImage9(),
            super::HourlyForecastImage10(),
        ]
    }

    pub(crate) fn daily_labels() -> [&'static gtk4::Label; 6] {
        [
            super::DailyForecastLabel1(),
            super::DailyForecastLabel2(),
            super::DailyForecastLabel3(),
            super::DailyForecastLabel4(),
            super::DailyForecastLabel5(),
            super::DailyForecastLabel6(),
        ]
    }

    pub(crate) fn daily_images() -> [&'static gtk4::Image; 6] {
        [
            super::DailyForecastImage1(),
            super::DailyForecastImage2(),
            super::DailyForecastImage3(),
            super::DailyForecastImage4(),
            super::DailyForecastImage5(),
            super::DailyForecastImage6(),
        ]
    }
}

pub(crate) mod cpu {
    pub(crate) fn labels() -> [&'static gtk4::Label; 12] {
        [
            super::CPUWidgetLabel1(),
            super::CPUWidgetLabel2(),
            super::CPUWidgetLabel3(),
            super::CPUWidgetLabel4(),
            super::CPUWidgetLabel5(),
            super::CPUWidgetLabel6(),
            super::CPUWidgetLabel7(),
            super::CPUWidgetLabel8(),
            super::CPUWidgetLabel9(),
            super::CPUWidgetLabel10(),
            super::CPUWidgetLabel11(),
            super::CPUWidgetLabel12(),
        ]
    }
}

pub(crate) mod workspaces {
    pub(crate) fn buttons() -> [&'static gtk4::Button; 10] {
        [
            super::WorkspacesWidgetButton1(),
            super::WorkspacesWidgetButton2(),
            super::WorkspacesWidgetButton3(),
            super::WorkspacesWidgetButton4(),
            super::WorkspacesWidgetButton5(),
            super::WorkspacesWidgetButton6(),
            super::WorkspacesWidgetButton7(),
            super::WorkspacesWidgetButton8(),
            super::WorkspacesWidgetButton9(),
            super::WorkspacesWidgetButton10(),
        ]
    }
}

macro_rules! widget {
    ($name:ident, $t:ty) => {
        paste::paste! {
            static mut [< $name _VALUE >]: Option<$t> = None;

            fn [< load_ $name >](builder: &gtk4::Builder) {
                unsafe {
                    [< $name _VALUE >] = builder.object(stringify!($name));
                }
            }

            pub(crate) fn $name() -> &'static $t {
                unsafe {
                    [< $name _VALUE >].as_mut().unwrap()
                }
            }
        }
    };
}

pub(crate) use widget;

#[allow(non_snake_case, non_upper_case_globals)]
mod load;
pub(crate) use load::*;

pub(crate) fn load() {
    const UI: &str = include_str!("../../../Widgets.ui");
    let builder = gtk4::Builder::from_string(UI);

    load_widgets(&builder);
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
            super::Hourly1(),
            super::Hourly2(),
            super::Hourly3(),
            super::Hourly4(),
            super::Hourly5(),
            super::Hourly6(),
            super::Hourly7(),
            super::Hourly8(),
            super::Hourly9(),
            super::Hourly10(),
        ]
    }

    pub(crate) fn daily_labels() -> [&'static gtk4::Label; 6] {
        [
            super::Daily1(),
            super::Daily2(),
            super::Daily3(),
            super::Daily4(),
            super::Daily5(),
            super::Daily6(),
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

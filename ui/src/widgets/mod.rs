#[allow(non_snake_case)]
mod gen;
pub(crate) use gen::*;

pub(crate) mod weather;

pub(crate) fn load() {
    const UI: &str = include_str!("../../Widgets.ui");
    let builder = gtk4::Builder::from_string(UI);

    unsafe { init_widgets(&builder) }

    weather::setup();
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

macro_rules! widget {
    ($name:ident, $t:ty) => {
        paste::paste! {
            #[allow(non_upper_case_globals)]
            static mut [< $name Instance >]: Option<$t> = None;

            #[allow(non_snake_case)]
            pub(crate) fn $name() -> &'static mut $t {
                unsafe {
                    match [< $name Instance >].as_mut() {
                        Some(value) => value,
                        None => {
                            eprintln!("widget {} is not defined", stringify!($name));
                            std::process::exit(1);
                        }
                    }
                }
            }

            #[allow(non_snake_case)]
            fn [< set_ $name >](v: $t) {
                unsafe { [< $name Instance >] = Some(v) }
            }
        }
    };
}
pub(crate) use widget;

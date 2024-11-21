static mut TOPBARWINDOW: Option<gtk4::Window> = None;
pub(crate) fn TopBarWindow() -> &'static gtk4::Window {
    unsafe {
        match TOPBARWINDOW.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget TopBarWindow is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WORKSPACESWIDGETBUTTON1: Option<gtk4::Button> = None;
pub(crate) fn WorkspacesWidgetButton1() -> &'static gtk4::Button {
    unsafe {
        match WORKSPACESWIDGETBUTTON1.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WorkspacesWidgetButton1 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WORKSPACESWIDGETBUTTON2: Option<gtk4::Button> = None;
pub(crate) fn WorkspacesWidgetButton2() -> &'static gtk4::Button {
    unsafe {
        match WORKSPACESWIDGETBUTTON2.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WorkspacesWidgetButton2 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WORKSPACESWIDGETBUTTON3: Option<gtk4::Button> = None;
pub(crate) fn WorkspacesWidgetButton3() -> &'static gtk4::Button {
    unsafe {
        match WORKSPACESWIDGETBUTTON3.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WorkspacesWidgetButton3 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WORKSPACESWIDGETBUTTON4: Option<gtk4::Button> = None;
pub(crate) fn WorkspacesWidgetButton4() -> &'static gtk4::Button {
    unsafe {
        match WORKSPACESWIDGETBUTTON4.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WorkspacesWidgetButton4 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WORKSPACESWIDGETBUTTON5: Option<gtk4::Button> = None;
pub(crate) fn WorkspacesWidgetButton5() -> &'static gtk4::Button {
    unsafe {
        match WORKSPACESWIDGETBUTTON5.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WorkspacesWidgetButton5 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WORKSPACESWIDGETBUTTON6: Option<gtk4::Button> = None;
pub(crate) fn WorkspacesWidgetButton6() -> &'static gtk4::Button {
    unsafe {
        match WORKSPACESWIDGETBUTTON6.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WorkspacesWidgetButton6 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WORKSPACESWIDGETBUTTON7: Option<gtk4::Button> = None;
pub(crate) fn WorkspacesWidgetButton7() -> &'static gtk4::Button {
    unsafe {
        match WORKSPACESWIDGETBUTTON7.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WorkspacesWidgetButton7 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WORKSPACESWIDGETBUTTON8: Option<gtk4::Button> = None;
pub(crate) fn WorkspacesWidgetButton8() -> &'static gtk4::Button {
    unsafe {
        match WORKSPACESWIDGETBUTTON8.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WorkspacesWidgetButton8 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WORKSPACESWIDGETBUTTON9: Option<gtk4::Button> = None;
pub(crate) fn WorkspacesWidgetButton9() -> &'static gtk4::Button {
    unsafe {
        match WORKSPACESWIDGETBUTTON9.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WorkspacesWidgetButton9 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WORKSPACESWIDGETBUTTON10: Option<gtk4::Button> = None;
pub(crate) fn WorkspacesWidgetButton10() -> &'static gtk4::Button {
    unsafe {
        match WORKSPACESWIDGETBUTTON10.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WorkspacesWidgetButton10 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut HTOPWIDGET: Option<gtk4::Button> = None;
pub(crate) fn HtopWidget() -> &'static gtk4::Button {
    unsafe {
        match HTOPWIDGET.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget HtopWidget is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WEATHERWIDGET: Option<gtk4::Button> = None;
pub(crate) fn WeatherWidget() -> &'static gtk4::Button {
    unsafe {
        match WEATHERWIDGET.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WeatherWidget is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WEATHERWIDGETLABEL: Option<gtk4::Label> = None;
pub(crate) fn WeatherWidgetLabel() -> &'static gtk4::Label {
    unsafe {
        match WEATHERWIDGETLABEL.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WeatherWidgetLabel is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LANGUAGEWIDGETLABEL: Option<gtk4::Label> = None;
pub(crate) fn LanguageWidgetLabel() -> &'static gtk4::Label {
    unsafe {
        match LANGUAGEWIDGETLABEL.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LanguageWidgetLabel is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut SOUNDWIDGETIMAGE: Option<gtk4::Image> = None;
pub(crate) fn SoundWidgetImage() -> &'static gtk4::Image {
    unsafe {
        match SOUNDWIDGETIMAGE.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget SoundWidgetImage is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut SOUNDWIDGETSCALE: Option<gtk4::Scale> = None;
pub(crate) fn SoundWidgetScale() -> &'static gtk4::Scale {
    unsafe {
        match SOUNDWIDGETSCALE.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget SoundWidgetScale is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CPUWIDGETLABEL1: Option<gtk4::Label> = None;
pub(crate) fn CPUWidgetLabel1() -> &'static gtk4::Label {
    unsafe {
        match CPUWIDGETLABEL1.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget CPUWidgetLabel1 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CPUWIDGETLABEL2: Option<gtk4::Label> = None;
pub(crate) fn CPUWidgetLabel2() -> &'static gtk4::Label {
    unsafe {
        match CPUWIDGETLABEL2.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget CPUWidgetLabel2 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CPUWIDGETLABEL3: Option<gtk4::Label> = None;
pub(crate) fn CPUWidgetLabel3() -> &'static gtk4::Label {
    unsafe {
        match CPUWIDGETLABEL3.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget CPUWidgetLabel3 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CPUWIDGETLABEL4: Option<gtk4::Label> = None;
pub(crate) fn CPUWidgetLabel4() -> &'static gtk4::Label {
    unsafe {
        match CPUWIDGETLABEL4.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget CPUWidgetLabel4 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CPUWIDGETLABEL5: Option<gtk4::Label> = None;
pub(crate) fn CPUWidgetLabel5() -> &'static gtk4::Label {
    unsafe {
        match CPUWIDGETLABEL5.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget CPUWidgetLabel5 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CPUWIDGETLABEL6: Option<gtk4::Label> = None;
pub(crate) fn CPUWidgetLabel6() -> &'static gtk4::Label {
    unsafe {
        match CPUWIDGETLABEL6.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget CPUWidgetLabel6 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CPUWIDGETLABEL7: Option<gtk4::Label> = None;
pub(crate) fn CPUWidgetLabel7() -> &'static gtk4::Label {
    unsafe {
        match CPUWIDGETLABEL7.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget CPUWidgetLabel7 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CPUWIDGETLABEL8: Option<gtk4::Label> = None;
pub(crate) fn CPUWidgetLabel8() -> &'static gtk4::Label {
    unsafe {
        match CPUWIDGETLABEL8.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget CPUWidgetLabel8 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CPUWIDGETLABEL9: Option<gtk4::Label> = None;
pub(crate) fn CPUWidgetLabel9() -> &'static gtk4::Label {
    unsafe {
        match CPUWIDGETLABEL9.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget CPUWidgetLabel9 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CPUWIDGETLABEL10: Option<gtk4::Label> = None;
pub(crate) fn CPUWidgetLabel10() -> &'static gtk4::Label {
    unsafe {
        match CPUWIDGETLABEL10.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget CPUWidgetLabel10 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CPUWIDGETLABEL11: Option<gtk4::Label> = None;
pub(crate) fn CPUWidgetLabel11() -> &'static gtk4::Label {
    unsafe {
        match CPUWIDGETLABEL11.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget CPUWidgetLabel11 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CPUWIDGETLABEL12: Option<gtk4::Label> = None;
pub(crate) fn CPUWidgetLabel12() -> &'static gtk4::Label {
    unsafe {
        match CPUWIDGETLABEL12.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget CPUWidgetLabel12 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut RAMWIDGET: Option<gtk4::Button> = None;
pub(crate) fn RAMWidget() -> &'static gtk4::Button {
    unsafe {
        match RAMWIDGET.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget RAMWidget is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut RAMWIDGETLABEL: Option<gtk4::Label> = None;
pub(crate) fn RAMWidgetLabel() -> &'static gtk4::Label {
    unsafe {
        match RAMWIDGETLABEL.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget RAMWidgetLabel is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut NETWORKWIDGET: Option<gtk4::Button> = None;
pub(crate) fn NetworkWidget() -> &'static gtk4::Button {
    unsafe {
        match NETWORKWIDGET.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget NetworkWidget is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut NETWORKWIDGETLABEL: Option<gtk4::Label> = None;
pub(crate) fn NetworkWidgetLabel() -> &'static gtk4::Label {
    unsafe {
        match NETWORKWIDGETLABEL.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget NetworkWidgetLabel is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut NETWORKWIDGETIMAGE: Option<gtk4::Image> = None;
pub(crate) fn NetworkWidgetImage() -> &'static gtk4::Image {
    unsafe {
        match NETWORKWIDGETIMAGE.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget NetworkWidgetImage is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut CLOCKWIDGETLABEL: Option<gtk4::Label> = None;
pub(crate) fn ClockWidgetLabel() -> &'static gtk4::Label {
    unsafe {
        match CLOCKWIDGETLABEL.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget ClockWidgetLabel is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut POWERWIDGET: Option<gtk4::Button> = None;
pub(crate) fn PowerWidget() -> &'static gtk4::Button {
    unsafe {
        match POWERWIDGET.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget PowerWidget is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut POWERWIDGETIMAGE: Option<gtk4::Image> = None;
pub(crate) fn PowerWidgetImage() -> &'static gtk4::Image {
    unsafe {
        match POWERWIDGETIMAGE.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget PowerWidgetImage is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LOGOUTSCREENWINDOW: Option<gtk4::Window> = None;
pub(crate) fn LogoutScreenWindow() -> &'static gtk4::Window {
    unsafe {
        match LOGOUTSCREENWINDOW.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LogoutScreenWindow is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LOGOUTSCREENLOCKBUTTON: Option<gtk4::Button> = None;
pub(crate) fn LogoutScreenLockButton() -> &'static gtk4::Button {
    unsafe {
        match LOGOUTSCREENLOCKBUTTON.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LogoutScreenLockButton is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LOGOUTSCREENREBOOTBUTTON: Option<gtk4::Button> = None;
pub(crate) fn LogoutScreenRebootButton() -> &'static gtk4::Button {
    unsafe {
        match LOGOUTSCREENREBOOTBUTTON.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LogoutScreenRebootButton is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LOGOUTSCREENSHUTDOWNBUTTON: Option<gtk4::Button> = None;
pub(crate) fn LogoutScreenShutdownButton() -> &'static gtk4::Button {
    unsafe {
        match LOGOUTSCREENSHUTDOWNBUTTON.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LogoutScreenShutdownButton is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LOGOUTSCREENLOGOUTBUTTON: Option<gtk4::Button> = None;
pub(crate) fn LogoutScreenLogoutButton() -> &'static gtk4::Button {
    unsafe {
        match LOGOUTSCREENLOGOUTBUTTON.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LogoutScreenLogoutButton is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut NETWORKSWINDOW: Option<gtk4::Window> = None;
pub(crate) fn NetworksWindow() -> &'static gtk4::Window {
    unsafe {
        match NETWORKSWINDOW.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget NetworksWindow is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut NETWORKROW1: Option<gtk4::CenterBox> = None;
pub(crate) fn NetworkRow1() -> &'static gtk4::CenterBox {
    unsafe {
        match NETWORKROW1.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget NetworkRow1 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut NETWORKROW2: Option<gtk4::CenterBox> = None;
pub(crate) fn NetworkRow2() -> &'static gtk4::CenterBox {
    unsafe {
        match NETWORKROW2.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget NetworkRow2 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut NETWORKROW3: Option<gtk4::CenterBox> = None;
pub(crate) fn NetworkRow3() -> &'static gtk4::CenterBox {
    unsafe {
        match NETWORKROW3.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget NetworkRow3 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut NETWORKROW4: Option<gtk4::CenterBox> = None;
pub(crate) fn NetworkRow4() -> &'static gtk4::CenterBox {
    unsafe {
        match NETWORKROW4.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget NetworkRow4 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut NETWORKROW5: Option<gtk4::CenterBox> = None;
pub(crate) fn NetworkRow5() -> &'static gtk4::CenterBox {
    unsafe {
        match NETWORKROW5.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget NetworkRow5 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut NETWORKSETTINGSROW: Option<gtk4::CenterBox> = None;
pub(crate) fn NetworkSettingsRow() -> &'static gtk4::CenterBox {
    unsafe {
        match NETWORKSETTINGSROW.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget NetworkSettingsRow is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut NETWORKEXITROW: Option<gtk4::CenterBox> = None;
pub(crate) fn NetworkExitRow() -> &'static gtk4::CenterBox {
    unsafe {
        match NETWORKEXITROW.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget NetworkExitRow is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERWINDOW: Option<gtk4::Window> = None;
pub(crate) fn LauncherWindow() -> &'static gtk4::Window {
    unsafe {
        match LAUNCHERWINDOW.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherWindow is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERENTRY: Option<gtk4::SearchEntry> = None;
pub(crate) fn LauncherEntry() -> &'static gtk4::SearchEntry {
    unsafe {
        match LAUNCHERENTRY.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherEntry is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW1: Option<gtk4::Box> = None;
pub(crate) fn LauncherRow1() -> &'static gtk4::Box {
    unsafe {
        match LAUNCHERROW1.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow1 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW1IMAGE: Option<gtk4::Image> = None;
pub(crate) fn LauncherRow1Image() -> &'static gtk4::Image {
    unsafe {
        match LAUNCHERROW1IMAGE.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow1Image is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW1LABEL: Option<gtk4::Label> = None;
pub(crate) fn LauncherRow1Label() -> &'static gtk4::Label {
    unsafe {
        match LAUNCHERROW1LABEL.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow1Label is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW2: Option<gtk4::Box> = None;
pub(crate) fn LauncherRow2() -> &'static gtk4::Box {
    unsafe {
        match LAUNCHERROW2.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow2 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW2IMAGE: Option<gtk4::Image> = None;
pub(crate) fn LauncherRow2Image() -> &'static gtk4::Image {
    unsafe {
        match LAUNCHERROW2IMAGE.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow2Image is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW2LABEL: Option<gtk4::Label> = None;
pub(crate) fn LauncherRow2Label() -> &'static gtk4::Label {
    unsafe {
        match LAUNCHERROW2LABEL.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow2Label is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW3: Option<gtk4::Box> = None;
pub(crate) fn LauncherRow3() -> &'static gtk4::Box {
    unsafe {
        match LAUNCHERROW3.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow3 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW3IMAGE: Option<gtk4::Image> = None;
pub(crate) fn LauncherRow3Image() -> &'static gtk4::Image {
    unsafe {
        match LAUNCHERROW3IMAGE.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow3Image is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW3LABEL: Option<gtk4::Label> = None;
pub(crate) fn LauncherRow3Label() -> &'static gtk4::Label {
    unsafe {
        match LAUNCHERROW3LABEL.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow3Label is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW4: Option<gtk4::Box> = None;
pub(crate) fn LauncherRow4() -> &'static gtk4::Box {
    unsafe {
        match LAUNCHERROW4.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow4 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW4IMAGE: Option<gtk4::Image> = None;
pub(crate) fn LauncherRow4Image() -> &'static gtk4::Image {
    unsafe {
        match LAUNCHERROW4IMAGE.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow4Image is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW4LABEL: Option<gtk4::Label> = None;
pub(crate) fn LauncherRow4Label() -> &'static gtk4::Label {
    unsafe {
        match LAUNCHERROW4LABEL.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow4Label is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW5: Option<gtk4::Box> = None;
pub(crate) fn LauncherRow5() -> &'static gtk4::Box {
    unsafe {
        match LAUNCHERROW5.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow5 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW5IMAGE: Option<gtk4::Image> = None;
pub(crate) fn LauncherRow5Image() -> &'static gtk4::Image {
    unsafe {
        match LAUNCHERROW5IMAGE.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow5Image is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut LAUNCHERROW5LABEL: Option<gtk4::Label> = None;
pub(crate) fn LauncherRow5Label() -> &'static gtk4::Label {
    unsafe {
        match LAUNCHERROW5LABEL.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget LauncherRow5Label is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut HTOPWINDOW: Option<gtk4::Window> = None;
pub(crate) fn HtopWindow() -> &'static gtk4::Window {
    unsafe {
        match HTOPWINDOW.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget HtopWindow is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WEATHERWINDOW: Option<gtk4::Window> = None;
pub(crate) fn WeatherWindow() -> &'static gtk4::Window {
    unsafe {
        match WEATHERWINDOW.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget WeatherWindow is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut HOURLY1: Option<gtk4::Label> = None;
pub(crate) fn Hourly1() -> &'static gtk4::Label {
    unsafe {
        match HOURLY1.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Hourly1 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut HOURLY2: Option<gtk4::Label> = None;
pub(crate) fn Hourly2() -> &'static gtk4::Label {
    unsafe {
        match HOURLY2.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Hourly2 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut HOURLY3: Option<gtk4::Label> = None;
pub(crate) fn Hourly3() -> &'static gtk4::Label {
    unsafe {
        match HOURLY3.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Hourly3 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut HOURLY4: Option<gtk4::Label> = None;
pub(crate) fn Hourly4() -> &'static gtk4::Label {
    unsafe {
        match HOURLY4.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Hourly4 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut HOURLY5: Option<gtk4::Label> = None;
pub(crate) fn Hourly5() -> &'static gtk4::Label {
    unsafe {
        match HOURLY5.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Hourly5 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut HOURLY6: Option<gtk4::Label> = None;
pub(crate) fn Hourly6() -> &'static gtk4::Label {
    unsafe {
        match HOURLY6.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Hourly6 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut HOURLY7: Option<gtk4::Label> = None;
pub(crate) fn Hourly7() -> &'static gtk4::Label {
    unsafe {
        match HOURLY7.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Hourly7 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut HOURLY8: Option<gtk4::Label> = None;
pub(crate) fn Hourly8() -> &'static gtk4::Label {
    unsafe {
        match HOURLY8.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Hourly8 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut HOURLY9: Option<gtk4::Label> = None;
pub(crate) fn Hourly9() -> &'static gtk4::Label {
    unsafe {
        match HOURLY9.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Hourly9 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut HOURLY10: Option<gtk4::Label> = None;
pub(crate) fn Hourly10() -> &'static gtk4::Label {
    unsafe {
        match HOURLY10.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Hourly10 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut DAILY1: Option<gtk4::Label> = None;
pub(crate) fn Daily1() -> &'static gtk4::Label {
    unsafe {
        match DAILY1.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Daily1 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut DAILY2: Option<gtk4::Label> = None;
pub(crate) fn Daily2() -> &'static gtk4::Label {
    unsafe {
        match DAILY2.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Daily2 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut DAILY3: Option<gtk4::Label> = None;
pub(crate) fn Daily3() -> &'static gtk4::Label {
    unsafe {
        match DAILY3.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Daily3 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut DAILY4: Option<gtk4::Label> = None;
pub(crate) fn Daily4() -> &'static gtk4::Label {
    unsafe {
        match DAILY4.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Daily4 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut DAILY5: Option<gtk4::Label> = None;
pub(crate) fn Daily5() -> &'static gtk4::Label {
    unsafe {
        match DAILY5.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Daily5 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut DAILY6: Option<gtk4::Label> = None;
pub(crate) fn Daily6() -> &'static gtk4::Label {
    unsafe {
        match DAILY6.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("widget Daily6 is not initialised");
                std::process::exit(1);
            }
        }
    }
}
pub(crate) unsafe fn init_widgets(builder: &gtk4::Builder) {
    TOPBARWINDOW = builder.object("TopBarWindow");
    WORKSPACESWIDGETBUTTON1 = builder.object("WorkspacesWidgetButton1");
    WORKSPACESWIDGETBUTTON2 = builder.object("WorkspacesWidgetButton2");
    WORKSPACESWIDGETBUTTON3 = builder.object("WorkspacesWidgetButton3");
    WORKSPACESWIDGETBUTTON4 = builder.object("WorkspacesWidgetButton4");
    WORKSPACESWIDGETBUTTON5 = builder.object("WorkspacesWidgetButton5");
    WORKSPACESWIDGETBUTTON6 = builder.object("WorkspacesWidgetButton6");
    WORKSPACESWIDGETBUTTON7 = builder.object("WorkspacesWidgetButton7");
    WORKSPACESWIDGETBUTTON8 = builder.object("WorkspacesWidgetButton8");
    WORKSPACESWIDGETBUTTON9 = builder.object("WorkspacesWidgetButton9");
    WORKSPACESWIDGETBUTTON10 = builder.object("WorkspacesWidgetButton10");
    HTOPWIDGET = builder.object("HtopWidget");
    WEATHERWIDGET = builder.object("WeatherWidget");
    WEATHERWIDGETLABEL = builder.object("WeatherWidgetLabel");
    LANGUAGEWIDGETLABEL = builder.object("LanguageWidgetLabel");
    SOUNDWIDGETIMAGE = builder.object("SoundWidgetImage");
    SOUNDWIDGETSCALE = builder.object("SoundWidgetScale");
    CPUWIDGETLABEL1 = builder.object("CPUWidgetLabel1");
    CPUWIDGETLABEL2 = builder.object("CPUWidgetLabel2");
    CPUWIDGETLABEL3 = builder.object("CPUWidgetLabel3");
    CPUWIDGETLABEL4 = builder.object("CPUWidgetLabel4");
    CPUWIDGETLABEL5 = builder.object("CPUWidgetLabel5");
    CPUWIDGETLABEL6 = builder.object("CPUWidgetLabel6");
    CPUWIDGETLABEL7 = builder.object("CPUWidgetLabel7");
    CPUWIDGETLABEL8 = builder.object("CPUWidgetLabel8");
    CPUWIDGETLABEL9 = builder.object("CPUWidgetLabel9");
    CPUWIDGETLABEL10 = builder.object("CPUWidgetLabel10");
    CPUWIDGETLABEL11 = builder.object("CPUWidgetLabel11");
    CPUWIDGETLABEL12 = builder.object("CPUWidgetLabel12");
    RAMWIDGET = builder.object("RAMWidget");
    RAMWIDGETLABEL = builder.object("RAMWidgetLabel");
    NETWORKWIDGET = builder.object("NetworkWidget");
    NETWORKWIDGETLABEL = builder.object("NetworkWidgetLabel");
    NETWORKWIDGETIMAGE = builder.object("NetworkWidgetImage");
    CLOCKWIDGETLABEL = builder.object("ClockWidgetLabel");
    POWERWIDGET = builder.object("PowerWidget");
    POWERWIDGETIMAGE = builder.object("PowerWidgetImage");
    LOGOUTSCREENWINDOW = builder.object("LogoutScreenWindow");
    LOGOUTSCREENLOCKBUTTON = builder.object("LogoutScreenLockButton");
    LOGOUTSCREENREBOOTBUTTON = builder.object("LogoutScreenRebootButton");
    LOGOUTSCREENSHUTDOWNBUTTON = builder.object("LogoutScreenShutdownButton");
    LOGOUTSCREENLOGOUTBUTTON = builder.object("LogoutScreenLogoutButton");
    NETWORKSWINDOW = builder.object("NetworksWindow");
    NETWORKROW1 = builder.object("NetworkRow1");
    NETWORKROW2 = builder.object("NetworkRow2");
    NETWORKROW3 = builder.object("NetworkRow3");
    NETWORKROW4 = builder.object("NetworkRow4");
    NETWORKROW5 = builder.object("NetworkRow5");
    NETWORKSETTINGSROW = builder.object("NetworkSettingsRow");
    NETWORKEXITROW = builder.object("NetworkExitRow");
    LAUNCHERWINDOW = builder.object("LauncherWindow");
    LAUNCHERENTRY = builder.object("LauncherEntry");
    LAUNCHERROW1 = builder.object("LauncherRow1");
    LAUNCHERROW1IMAGE = builder.object("LauncherRow1Image");
    LAUNCHERROW1LABEL = builder.object("LauncherRow1Label");
    LAUNCHERROW2 = builder.object("LauncherRow2");
    LAUNCHERROW2IMAGE = builder.object("LauncherRow2Image");
    LAUNCHERROW2LABEL = builder.object("LauncherRow2Label");
    LAUNCHERROW3 = builder.object("LauncherRow3");
    LAUNCHERROW3IMAGE = builder.object("LauncherRow3Image");
    LAUNCHERROW3LABEL = builder.object("LauncherRow3Label");
    LAUNCHERROW4 = builder.object("LauncherRow4");
    LAUNCHERROW4IMAGE = builder.object("LauncherRow4Image");
    LAUNCHERROW4LABEL = builder.object("LauncherRow4Label");
    LAUNCHERROW5 = builder.object("LauncherRow5");
    LAUNCHERROW5IMAGE = builder.object("LauncherRow5Image");
    LAUNCHERROW5LABEL = builder.object("LauncherRow5Label");
    HTOPWINDOW = builder.object("HtopWindow");
    WEATHERWINDOW = builder.object("WeatherWindow");
    HOURLY1 = builder.object("Hourly1");
    HOURLY2 = builder.object("Hourly2");
    HOURLY3 = builder.object("Hourly3");
    HOURLY4 = builder.object("Hourly4");
    HOURLY5 = builder.object("Hourly5");
    HOURLY6 = builder.object("Hourly6");
    HOURLY7 = builder.object("Hourly7");
    HOURLY8 = builder.object("Hourly8");
    HOURLY9 = builder.object("Hourly9");
    HOURLY10 = builder.object("Hourly10");
    DAILY1 = builder.object("Daily1");
    DAILY2 = builder.object("Daily2");
    DAILY3 = builder.object("Daily3");
    DAILY4 = builder.object("Daily4");
    DAILY5 = builder.object("Daily5");
    DAILY6 = builder.object("Daily6");
}

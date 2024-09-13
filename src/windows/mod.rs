use std::collections::HashMap;

mod top_bar;
use gtk4::Window;
pub(crate) use top_bar::TopBar;

mod launcher;
pub(crate) use launcher::Launcher;

mod logout_screen;
pub(crate) use logout_screen::LogoutScreen;

mod networks;
pub(crate) use networks::Networks;

static mut WINDOWS: Option<HashMap<&'static str, Window>> = None;

pub(crate) trait GloballyAccessibleWindow {
    const NAME: &str;

    fn set(window: Window) {
        unsafe {
            if WINDOWS.is_none() {
                WINDOWS = Some(HashMap::new())
            }

            let windows = WINDOWS.as_mut().unwrap();
            windows.insert(Self::NAME, window);
        }
    }

    fn get() -> &'static mut Window {
        unsafe {
            let windows = WINDOWS.as_mut().unwrap();
            windows.get_mut(Self::NAME).unwrap()
        }
    }
}

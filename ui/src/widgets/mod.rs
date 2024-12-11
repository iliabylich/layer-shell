pub(crate) mod htop;
pub(crate) mod launcher;
pub(crate) mod networks;
pub(crate) mod session;
pub(crate) mod top_bar;
pub(crate) mod weather;

pub(crate) fn load() {
    htop::setup();
    launcher::setup();
    networks::setup();
    session::setup();
    top_bar::setup();
    weather::setup();
}

macro_rules! widget {
    ($name:ident, $t:ty) => {
        paste::paste! {
            #[allow(non_upper_case_globals)]
            static mut [< $name Instance >]: Option<$t> = None;

            #[allow(non_snake_case)]
            pub(crate) fn $name() -> &'static $t {
                unsafe {
                    match [< $name Instance >].as_ref() {
                        Some(value) => value,
                        None => {
                            log::error!("widget {} is not defined", stringify!($name));
                            std::process::exit(1);
                        }
                    }
                }
            }

            #[allow(non_snake_case)]
            pub(crate) fn [< set_ $name >](v: $t) {
                unsafe { [< $name Instance >] = Some(v) }
            }
        }
    };
}
pub(crate) use widget;

macro_rules! icon {
    ($name:ident, $path:literal) => {
        paste::paste! {
            #[allow(non_upper_case_globals)]
            static mut [< $name Instance >]: Option<gtk4::gdk::Texture> = None;
            pub(crate) fn [< $name _icon >]() -> &'static gtk4::gdk::Texture {
                unsafe {
                    #[allow(static_mut_refs)]
                    match [< $name Instance >].as_ref() {
                        Some(v) => v,
                        None => {
                            log::error!("icon {} is not initialised", stringify!($name));
                            std::process::exit(1);
                        }
                    }
                }
            }
            fn [< init_ $name >]() {
                const BYTES: &[u8] = include_bytes!($path);
                let bytes = gtk4::glib::Bytes::from_static(BYTES);
                let texture = gtk4::gdk::Texture::from_bytes(&bytes).unwrap();
                unsafe { [< $name Instance >] = Some(texture); }
            }
        }
    };
}

icon!(power, "../../icons/power.png");
icon!(question_mark, "../../icons/question_mark.png");
icon!(wifi, "../../icons/wifi.png");

icon!(foggy, "../../icons/foggy.png");
icon!(partly_cloudy, "../../icons/partly_cloudy.png");
icon!(rainy, "../../icons/rainy.png");
icon!(sunny, "../../icons/sunny.png");
icon!(thunderstorm, "../../icons/thunderstorm.png");
icon!(snowy, "../../icons/snowy.png");

pub(crate) fn load() {
    init_power();
    init_question_mark();
    init_wifi();

    init_foggy();
    init_partly_cloudy();
    init_rainy();
    init_sunny();
    init_thunderstorm();
    init_snowy();
}

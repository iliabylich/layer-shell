use gtk4::{gdk::Texture, glib::Bytes};

static mut FOGGY: Option<Texture> = None;
pub(crate) fn foggy_icon() -> &'static Texture {
    unsafe {
        match FOGGY.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("icon foggy is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut PARTLY_CLOUDY: Option<Texture> = None;
pub(crate) fn partly_cloudy_icon() -> &'static Texture {
    unsafe {
        match PARTLY_CLOUDY.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("icon partly_cloudy is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut POWER: Option<Texture> = None;
pub(crate) fn power_icon() -> &'static Texture {
    unsafe {
        match POWER.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("icon power is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut RAINY: Option<Texture> = None;
pub(crate) fn rainy_icon() -> &'static Texture {
    unsafe {
        match RAINY.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("icon rainy is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut SUNNY: Option<Texture> = None;
pub(crate) fn sunny_icon() -> &'static Texture {
    unsafe {
        match SUNNY.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("icon sunny is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut THUNDERSTORM: Option<Texture> = None;
pub(crate) fn thunderstorm_icon() -> &'static Texture {
    unsafe {
        match THUNDERSTORM.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("icon thunderstorm is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WEATHER_SNOWY: Option<Texture> = None;
pub(crate) fn weather_snowy_icon() -> &'static Texture {
    unsafe {
        match WEATHER_SNOWY.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("icon weather_snowy is not initialised");
                std::process::exit(1);
            }
        }
    }
}
static mut WIFI: Option<Texture> = None;
pub(crate) fn wifi_icon() -> &'static Texture {
    unsafe {
        match WIFI.as_ref() {
            Some(v) => v,
            None => {
                eprintln!("icon wifi is not initialised");
                std::process::exit(1);
            }
        }
    }
}
pub(crate) unsafe fn init_icons() {
    const FOGGY_BYTES: &[u8] = include_bytes!("../../../icons/foggy.svg");
    let foggy_bytes = Bytes::from_static(FOGGY_BYTES);
    let foggy_texture = Texture::from_bytes(&foggy_bytes).unwrap();
    FOGGY = Some(foggy_texture);

    const PARTLY_CLOUDY_BYTES: &[u8] = include_bytes!("../../../icons/partly_cloudy.svg");
    let partly_cloudy_bytes = Bytes::from_static(PARTLY_CLOUDY_BYTES);
    let partly_cloudy_texture = Texture::from_bytes(&partly_cloudy_bytes).unwrap();
    PARTLY_CLOUDY = Some(partly_cloudy_texture);

    const POWER_BYTES: &[u8] = include_bytes!("../../../icons/power.svg");
    let power_bytes = Bytes::from_static(POWER_BYTES);
    let power_texture = Texture::from_bytes(&power_bytes).unwrap();
    POWER = Some(power_texture);

    const RAINY_BYTES: &[u8] = include_bytes!("../../../icons/rainy.svg");
    let rainy_bytes = Bytes::from_static(RAINY_BYTES);
    let rainy_texture = Texture::from_bytes(&rainy_bytes).unwrap();
    RAINY = Some(rainy_texture);

    const SUNNY_BYTES: &[u8] = include_bytes!("../../../icons/sunny.svg");
    let sunny_bytes = Bytes::from_static(SUNNY_BYTES);
    let sunny_texture = Texture::from_bytes(&sunny_bytes).unwrap();
    SUNNY = Some(sunny_texture);

    const THUNDERSTORM_BYTES: &[u8] = include_bytes!("../../../icons/thunderstorm.svg");
    let thunderstorm_bytes = Bytes::from_static(THUNDERSTORM_BYTES);
    let thunderstorm_texture = Texture::from_bytes(&thunderstorm_bytes).unwrap();
    THUNDERSTORM = Some(thunderstorm_texture);

    const WEATHER_SNOWY_BYTES: &[u8] = include_bytes!("../../../icons/weather_snowy.svg");
    let weather_snowy_bytes = Bytes::from_static(WEATHER_SNOWY_BYTES);
    let weather_snowy_texture = Texture::from_bytes(&weather_snowy_bytes).unwrap();
    WEATHER_SNOWY = Some(weather_snowy_texture);

    const WIFI_BYTES: &[u8] = include_bytes!("../../../icons/wifi.svg");
    let wifi_bytes = Bytes::from_static(WIFI_BYTES);
    let wifi_texture = Texture::from_bytes(&wifi_bytes).unwrap();
    WIFI = Some(wifi_texture);

    ()
}

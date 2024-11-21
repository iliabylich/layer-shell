use gtk4::{gdk::Texture, glib::Bytes};

pub(crate) fn power_icon() -> Texture {
    const BYTES: &[u8] = include_bytes!("../../../icons/power.svg");
    let bytes = Bytes::from_static(BYTES);
    Texture::from_bytes(&bytes).unwrap()
}
pub(crate) fn wifi_icon() -> Texture {
    const BYTES: &[u8] = include_bytes!("../../../icons/wifi.svg");
    let bytes = Bytes::from_static(BYTES);
    Texture::from_bytes(&bytes).unwrap()
}

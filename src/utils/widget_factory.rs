use gtk4::{glib::Object, prelude::IsA, Builder};

static mut BUILDER: Option<Builder> = None;

const UI: &str = include_str!("../../Widgets.ui");

pub(crate) struct WidgetFactory;

impl WidgetFactory {
    pub(crate) fn init() {
        unsafe {
            BUILDER = Some(Builder::from_string(UI));
        }
    }

    fn get() -> Builder {
        unsafe { BUILDER.clone().unwrap() }
    }
}

pub(crate) fn load_widget<T: IsA<Object>>(name: &str) -> T {
    let builder = WidgetFactory::get();

    builder.object(name).unwrap()
}

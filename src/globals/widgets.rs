use std::collections::HashMap;

use gtk4::{
    glib::Object,
    prelude::{BuildableExt, Cast, IsA},
    Builder, Widget,
};

pub(crate) struct GlobalWidgets;

static mut WIDGET_TO_NAME: Option<HashMap<String, Widget>> = None;
fn widget_to_name_map() -> &'static mut HashMap<String, Widget> {
    unsafe { WIDGET_TO_NAME.as_mut().unwrap() }
}
const UI: &str = include_str!("../../Widgets.ui");

impl GlobalWidgets {
    pub(crate) fn init() {
        let builder = Builder::from_string(UI);
        unsafe {
            WIDGET_TO_NAME = Some(HashMap::new());
        }

        for object in builder.objects() {
            if let Ok(widget) = object.dynamic_cast::<Widget>() {
                if let Some(id) = widget.buildable_id() {
                    let id = id.to_string();
                    widget_to_name_map().insert(id, widget);
                }
            }
        }
    }
}

pub(crate) fn load_widget<T: IsA<Object>>(name: &str) -> &'static T {
    widget_to_name_map()
        .get(name)
        .unwrap_or_else(|| panic!("Can't find widget {name}"))
        .dynamic_cast_ref()
        .unwrap()
}

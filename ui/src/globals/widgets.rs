use gtk4::{
    glib::Object,
    prelude::{BuildableExt, Cast, IsA},
    Builder, Widget,
};
use layer_shell_utils::global;
use std::collections::HashMap;

pub(crate) struct GlobalWidgets;
global!(MAP, HashMap<String, Widget>);

impl GlobalWidgets {
    pub(crate) fn init() {
        let mut map = HashMap::new();

        const UI: &str = include_str!("../../../Widgets.ui");
        let builder = Builder::from_string(UI);

        for object in builder.objects() {
            if let Ok(widget) = object.dynamic_cast::<Widget>() {
                if let Some(id) = widget.buildable_id() {
                    let id = id.to_string();
                    map.insert(id, widget);
                }
            }
        }

        MAP::set(map);
    }
}

pub(crate) fn load_widget<T: IsA<Object>>(name: &str) -> &'static T {
    MAP::get()
        .get(name)
        .unwrap_or_else(|| panic!("Can't find widget {name}"))
        .dynamic_cast_ref()
        .expect("failed to cast")
}

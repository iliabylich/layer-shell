use std::collections::HashMap;

use gtk4::{
    glib::Object,
    prelude::{BuildableExt, Cast, IsA},
    Builder, Widget,
};

use crate::utils::{singleton, Singleton};

pub(crate) struct GlobalWidgets {
    map: HashMap<String, Widget>,
}
singleton!(GlobalWidgets);

impl GlobalWidgets {
    pub(crate) fn init() {
        let mut map = HashMap::new();

        const UI: &str = include_str!("../../Widgets.ui");
        let builder = Builder::from_string(UI);

        for object in builder.objects() {
            if let Ok(widget) = object.dynamic_cast::<Widget>() {
                if let Some(id) = widget.buildable_id() {
                    let id = id.to_string();
                    map.insert(id, widget);
                }
            }
        }

        Self::set(Self { map });
    }
}

pub(crate) fn load_widget<T: IsA<Object>>(name: &str) -> &'static T {
    GlobalWidgets::get()
        .map
        .get(name)
        .unwrap_or_else(|| panic!("Can't find widget {name}"))
        .dynamic_cast_ref()
        .unwrap()
}

pub(crate) fn globalize_widget<T: IsA<Object>>(w: &Widget) -> &'static T {
    let id = w.buildable_id().unwrap();
    load_widget(id.as_str())
}

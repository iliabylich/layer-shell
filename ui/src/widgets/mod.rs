macro_rules! widget {
    ($name:ident, $t:ty) => {
        paste::paste! {
            static mut [< $name _VALUE >]: Option<$t> = None;

            fn [< load_ $name >](builder: &gtk4::Builder) {
                unsafe {
                    [< $name _VALUE >] = builder.object(stringify!($name));
                }
            }

            pub(crate) fn $name() -> &'static $t {
                unsafe {
                    [< $name _VALUE >].as_mut().unwrap()
                }
            }
        }
    };
}

pub(crate) use widget;

#[allow(non_snake_case, non_upper_case_globals)]
mod load;
pub(crate) use load::*;

pub(crate) fn load() {
    const UI: &str = include_str!("../../../Widgets.ui");
    let builder = gtk4::Builder::from_string(UI);

    load_widgets(&builder);
}

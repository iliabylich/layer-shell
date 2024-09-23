use gtk4::{prelude::ButtonExt, Button, Label};

use crate::{
    globals::load_widget,
    models::Memory,
    utils::{exec_async, TypedChildren},
};

pub(crate) fn init() {
    let widget = load_widget::<Button>("RAMWidget");
    let label = widget.first_child_as::<Label>();

    Memory::spawn(|mem| {
        label.set_label(&format!("RAM {:.1}G/{:.1}G", mem.used, mem.total));
    });

    widget.connect_clicked(|_| {
        gtk4::glib::spawn_future_local(async {
            exec_async(&["gnome-system-monitor"]).await;
        });
    });
}

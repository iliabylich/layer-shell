use gtk4::{prelude::ButtonExt, Button, Label};

use crate::{
    models::Memory,
    utils::{exec_async, load_widget},
};

pub(crate) struct RAM;

impl RAM {
    pub(crate) fn init() {
        let widget: Button = load_widget("RAM");
        let label: Label = load_widget("RAMLabel");

        Memory::spawn(move |mem| {
            label.set_label(&format!("RAM {:.2}G/{:.2}G", mem.used, mem.total));
        });

        widget.connect_clicked(move |_| {
            gtk4::glib::spawn_future_local(async {
                exec_async(&["gnome-system-monitor"]).await;
            });
        });
    }
}

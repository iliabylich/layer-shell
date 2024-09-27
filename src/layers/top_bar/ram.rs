use crate::{
    globals::load_widget,
    models::{Memory, MemoryData},
    utils::exec_async,
};
use gtk4::{prelude::ButtonExt, Button, Label};

pub(crate) fn init() {
    let widget = load_widget::<Button>("RAMWidget");

    fn on_change(mem: MemoryData) {
        let label = load_widget::<Label>("RAMWidgetLabel");
        label.set_label(&format!("RAM {:.1}G/{:.1}G", mem.used, mem.total));
    }

    Memory::subscribe(on_change);

    widget.connect_clicked(|_| {
        gtk4::glib::spawn_future_local(async {
            exec_async(&["gnome-system-monitor"]).await;
        });
    });
}

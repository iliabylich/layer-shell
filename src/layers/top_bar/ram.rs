use crate::{
    globals::load_widget,
    models::{subscribe, Event},
    utils::exec_async,
};
use gtk4::{prelude::ButtonExt, Button, Label};

pub(crate) fn init() {
    subscribe(on_change);

    let button = load_widget::<Button>("RAMWidget");
    button.connect_clicked(|_| {
        gtk4::glib::spawn_future_local(async {
            exec_async(&["gnome-system-monitor"]).await;
        });
    });
}

fn on_change(event: &Event) {
    let Event::Memory { used, total } = event else {
        return;
    };
    let label = load_widget::<Label>("RAMWidgetLabel");
    label.set_label(&format!("RAM {:.1}G/{:.1}G", used, total));
}

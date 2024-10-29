use crate::globals::load_widget;
use gtk4::{prelude::ButtonExt, Button, Label};
use layer_shell_io::{publish, subscribe, Command, Event};

pub(crate) fn init() {
    subscribe(on_event);

    let button = load_widget::<Button>("RAMWidget");
    button.connect_clicked(|_| {
        publish(Command::SpawnSystemMonitor);
    });
}

fn on_event(event: &Event) {
    if let Event::Memory { used, total } = event {
        let label = load_widget::<Label>("RAMWidgetLabel");
        label.set_label(&format!("RAM {:.1}G/{:.1}G", used, total));
    }
}

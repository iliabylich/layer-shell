use crate::widgets::{RAMWidget, RAMWidgetLabel};
use gtk4::prelude::ButtonExt;
use layer_shell_io::{publish, subscribe, Command, Event};

pub(crate) fn init() {
    subscribe(on_event);

    RAMWidget().connect_clicked(|_| {
        publish(Command::SpawnSystemMonitor);
    });
}

fn on_event(event: &Event) {
    if let Event::Memory { used, total } = event {
        RAMWidgetLabel().set_label(&format!("RAM {:.1}G/{:.1}G", used, total));
    }
}

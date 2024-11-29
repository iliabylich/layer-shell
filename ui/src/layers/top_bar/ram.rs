use crate::widgets::top_bar::ram::{Label, Widget};
use gtk4::prelude::ButtonExt;
use layer_shell_io::{publish, subscribe, Command, Event};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::Memory { used, total } = event {
            Label().set_label(&format!("RAM {:.1}G/{:.1}G", used, total));
        }
    });

    Widget().connect_clicked(|_| {
        publish(Command::SpawnSystemMonitor);
    });
}

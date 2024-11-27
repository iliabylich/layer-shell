use crate::widgets::workspaces::buttons;
use gtk4::prelude::{ButtonExt, WidgetExt};
use layer_shell_io::{publish, subscribe, Command, Event};

pub(crate) fn init() {
    for (idx, button) in buttons().iter().enumerate() {
        button.connect_clicked(move |_| {
            publish(Command::GoToWorkspace(idx));
        });
    }

    subscribe(|event| {
        if let Event::Workspaces { ids, active_id } = event {
            for idx in 1..=10 {
                let button = buttons()[idx - 1];
                button.set_visible(ids.contains(&idx) || idx <= 5);
                const ACTIVE: &[&str] = &["active"];
                const INACTIVE: &[&str] = &["inactive"];
                button.set_css_classes(if idx == *active_id { ACTIVE } else { INACTIVE })
            }
        }
    });
}

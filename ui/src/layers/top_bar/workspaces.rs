use crate::widgets::top_bar::workspaces::Buttons;
use gtk4::prelude::{ButtonExt, WidgetExt};
use layer_shell_io::{
    hyptland::{HyprlandGoToWorkspace, Workspaces},
    publish, subscribe, Command, Event,
};

pub(crate) fn init() {
    for (idx, button) in Buttons().iter().enumerate() {
        button.connect_clicked(move |_| {
            publish(Command::HyprlandGoToWorkspace(HyprlandGoToWorkspace {
                idx,
            }));
        });
    }

    subscribe(|event| {
        if let Event::Workspaces(Workspaces { ids, active_id }) = event {
            for idx in 1..=10 {
                let button = &Buttons()[idx - 1];
                button.set_visible(ids.contains(&idx) || idx <= 5);
                const ACTIVE: &[&str] = &["active"];
                const INACTIVE: &[&str] = &["inactive"];
                button.set_css_classes(if idx == *active_id { ACTIVE } else { INACTIVE })
            }
        }
    });
}

use crate::widgets::top_bar::workspaces::Buttons;
use gtk4::prelude::{ButtonExt, WidgetExt};
use layer_shell_hyprland::HyprlandGoToWorkspace;
use layer_shell_io::{publish, subscribe, Command, Event};

pub(crate) fn init() {
    for (idx, button) in Buttons().iter().enumerate() {
        button.connect_clicked(move |_| {
            publish(Command::HyprlandGoToWorkspace(HyprlandGoToWorkspace {
                idx,
            }));
        });
    }

    subscribe(|event| {
        if let Event::Workspaces(event) = event {
            for idx in 1..=10 {
                let button = &Buttons()[idx - 1];
                button.set_visible(event.ids.contains(&idx) || idx <= 5);
                const ACTIVE: &[&str] = &["active"];
                const INACTIVE: &[&str] = &["inactive"];
                button.set_css_classes(if idx == event.active_id {
                    ACTIVE
                } else {
                    INACTIVE
                })
            }
        }
    });
}

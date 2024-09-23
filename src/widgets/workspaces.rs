use gtk4::{
    prelude::{ButtonExt, WidgetExt},
    Button,
};

use crate::{globals::load_widget, models::HyprlandWorkspaces, utils::TypedChildren};

pub(crate) struct Workspaces;

impl Workspaces {
    pub(crate) fn init(min_workspaces: usize) {
        let widget = load_widget::<gtk4::Box>("WorkspacesWidget");
        let buttons = widget.children_as::<10, Button>();

        HyprlandWorkspaces::spawn(min_workspaces, move |workspaces| {
            for (button, workspace) in buttons.iter().zip(workspaces.iter()) {
                button.set_visible(workspace.visible);
                button.set_css_classes(if workspace.active {
                    &["active"]
                } else {
                    &["inactive"]
                })
            }
        });

        for (idx, button) in buttons.iter().enumerate() {
            button.connect_clicked(move |_| {
                HyprlandWorkspaces::go_to(idx + 1);
            });
        }
    }
}

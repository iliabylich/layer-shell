use gtk4::{
    prelude::{ButtonExt, WidgetExt},
    Button,
};

use crate::{globals::load_widget, models::HyprlandWorkspaces};

pub(crate) struct Workspaces;

impl Workspaces {
    pub(crate) fn init(min_workspaces: usize) {
        let buttons = [
            load_widget::<Button>("WorkspaceButton1"),
            load_widget::<Button>("WorkspaceButton2"),
            load_widget::<Button>("WorkspaceButton3"),
            load_widget::<Button>("WorkspaceButton4"),
            load_widget::<Button>("WorkspaceButton5"),
            load_widget::<Button>("WorkspaceButton6"),
            load_widget::<Button>("WorkspaceButton7"),
            load_widget::<Button>("WorkspaceButton8"),
            load_widget::<Button>("WorkspaceButton9"),
            load_widget::<Button>("WorkspaceButton10"),
        ];

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

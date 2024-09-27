use crate::{globals::load_widget, models::HyprlandWorkspaces};
use gtk4::{
    prelude::{ButtonExt, WidgetExt},
    Button,
};

pub(crate) fn init(min_workspaces: usize) {
    let buttons = [
        load_widget::<Button>("WorkspacesWidgetButton1"),
        load_widget::<Button>("WorkspacesWidgetButton2"),
        load_widget::<Button>("WorkspacesWidgetButton3"),
        load_widget::<Button>("WorkspacesWidgetButton4"),
        load_widget::<Button>("WorkspacesWidgetButton5"),
        load_widget::<Button>("WorkspacesWidgetButton6"),
        load_widget::<Button>("WorkspacesWidgetButton7"),
        load_widget::<Button>("WorkspacesWidgetButton8"),
        load_widget::<Button>("WorkspacesWidgetButton9"),
        load_widget::<Button>("WorkspacesWidgetButton10"),
    ];

    HyprlandWorkspaces::subscribe(min_workspaces, move |workspaces| {
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

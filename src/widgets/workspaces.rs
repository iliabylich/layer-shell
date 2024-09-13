use gtk4::{
    prelude::{ButtonExt, WidgetExt},
    Button,
};

use crate::{models::HyprlandWorkspaces, utils::load_widget};

pub(crate) struct Workspaces;

static mut BUTTONS: Option<[Button; 10]> = None;
fn buttons() -> &'static [Button; 10] {
    unsafe { BUTTONS.as_ref().unwrap() }
}

impl Workspaces {
    pub(crate) fn init(min_workspaces: usize) {
        unsafe {
            BUTTONS = Some([
                load_widget("WorkspaceButton1"),
                load_widget("WorkspaceButton2"),
                load_widget("WorkspaceButton3"),
                load_widget("WorkspaceButton4"),
                load_widget("WorkspaceButton5"),
                load_widget("WorkspaceButton6"),
                load_widget("WorkspaceButton7"),
                load_widget("WorkspaceButton8"),
                load_widget("WorkspaceButton9"),
                load_widget("WorkspaceButton10"),
            ]);
        }

        HyprlandWorkspaces::spawn(min_workspaces, |workspaces| {
            for (button, workspace) in buttons().iter().zip(workspaces.iter()) {
                button.set_visible(workspace.visible);
                button.set_css_classes(if workspace.active {
                    &["active"]
                } else {
                    &["inactive"]
                })
            }
        });

        for (idx, button) in buttons().iter().enumerate() {
            button.connect_clicked(move |_| {
                HyprlandWorkspaces::go_to(idx + 1);
            });
        }
    }
}

use crate::widgets::{
    WorkspacesWidgetButton1, WorkspacesWidgetButton10, WorkspacesWidgetButton2,
    WorkspacesWidgetButton3, WorkspacesWidgetButton4, WorkspacesWidgetButton5,
    WorkspacesWidgetButton6, WorkspacesWidgetButton7, WorkspacesWidgetButton8,
    WorkspacesWidgetButton9,
};
use gtk4::{
    prelude::{ButtonExt, WidgetExt},
    Button,
};
use layer_shell_io::{publish, subscribe, Command, Event};

pub(crate) fn init() {
    for (idx, button) in buttons().iter().enumerate() {
        button.connect_clicked(move |_| {
            publish(Command::GoToWorkspace(idx));
        });
    }

    subscribe(on_event);
}

fn on_event(event: &Event) {
    if let Event::Workspaces { ids, active_id } = event {
        for idx in 1..=10 {
            let button = buttons()[idx - 1];
            button.set_visible(ids.contains(&idx) || idx <= 5);
            button.set_css_classes(if idx == *active_id {
                &["active"]
            } else {
                &["inactive"]
            })
        }
    }
}

fn buttons() -> [&'static Button; 10] {
    [
        WorkspacesWidgetButton1(),
        WorkspacesWidgetButton2(),
        WorkspacesWidgetButton3(),
        WorkspacesWidgetButton4(),
        WorkspacesWidgetButton5(),
        WorkspacesWidgetButton6(),
        WorkspacesWidgetButton7(),
        WorkspacesWidgetButton8(),
        WorkspacesWidgetButton9(),
        WorkspacesWidgetButton10(),
    ]
}

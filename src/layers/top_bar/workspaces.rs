use crate::{
    globals::load_widget,
    models::{publish, subscribe, Command, Event},
};
use gtk4::{
    prelude::{ButtonExt, WidgetExt},
    Button,
};

pub(crate) fn init() {
    for (idx, button) in buttons().iter().enumerate() {
        button.connect_clicked(move |_| {
            publish(Command::GoToWorkspace { idx });
        });
    }

    subscribe(on_change);
}

fn on_change(event: &Event) {
    if let Event::Workspaces { ids, active_id } = event {
        let buttons = buttons();
        for idx in 1..=10 {
            let button = buttons[idx - 1];
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
    ]
}

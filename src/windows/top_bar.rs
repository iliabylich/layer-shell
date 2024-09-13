use gtk4::{prelude::GtkWindowExt, Application, Window};

use crate::{
    utils::{layer_window, load_widget, LayerOptions},
    widgets::{Language, Workspaces, CPU},
};

pub(crate) struct TopBar;

static mut TOP_BAR: Option<Window> = None;

impl TopBar {
    pub(crate) fn activate(app: &Application) {
        let window: Window = load_widget("TopBar");
        window.set_application(Some(app));
        layer_window(
            &window,
            LayerOptions::builder()
                .with_namespace("TopBar")
                .with_layer(gtk4_layer_shell::Layer::Overlay)
                .with_anchors(&[
                    gtk4_layer_shell::Edge::Top,
                    gtk4_layer_shell::Edge::Left,
                    gtk4_layer_shell::Edge::Right,
                ])
                .with_margins(&[(gtk4_layer_shell::Edge::Top, 0)])
                .build(),
        );

        Workspaces::init(5);
        // Terminal();
        Language::init();
        // Sound();
        CPU::init();
        // RAM();
        // WiFi();
        // Clock({ format: "%H:%M:%S", tooltipFormat: "%Y %B %e\n%A" });
        // PowerButton();

        window.present();

        unsafe { TOP_BAR = Some(window) };
    }

    pub(crate) fn get() -> Window {
        unsafe { TOP_BAR.clone().unwrap() }
    }
}

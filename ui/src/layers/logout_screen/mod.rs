use crate::utils::{keybindings, LayerWindow};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use layer_shell_io::{subscribe, Event};
use layer_shell_utils::global;

mod buttons;

pub(crate) struct LogoutScreen;
global!(RESET, Box<dyn Fn()>);

impl LayerWindow for LogoutScreen {
    const NAME: &'static str = "LogoutScreen";
    const LAYER: Layer = Layer::Overlay;
    const ANCHORS: &'static [Edge] = &[Edge::Top, Edge::Right, Edge::Bottom, Edge::Left];
    const MARGINS: &'static [(Edge, i32)] = &[];
    const KEYBOARD_MODE: Option<KeyboardMode> = Some(KeyboardMode::Exclusive);

    fn reset() {
        (RESET::get())()
    }
}

impl LogoutScreen {
    pub(crate) fn activate(app: &Application) {
        let window = Self::layer_window(app);

        let (reset, on_key_press) = buttons::init();

        keybindings(window)
            .add("Escape", Self::toggle)
            .fallback(on_key_press)
            .finish();

        window.present();
        window.set_visible(false);

        subscribe(on_event);

        RESET::set(reset)
    }
}

fn on_event(event: &Event) {
    if let Event::ToggleLogoutScreen = event {
        LogoutScreen::toggle();
    }
}

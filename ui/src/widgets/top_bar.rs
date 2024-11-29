use crate::widgets::widget;
use gtk4::prelude::{BoxExt, GtkWindowExt, WidgetExt};

widget!(TopBarWindow, gtk4::Window);

pub(crate) mod workspaces {
    use super::*;

    widget!(Widget, gtk4::Box);
    const SIZE: usize = 10;
    widget!(Buttons, [gtk4::Button; SIZE]);

    pub(crate) fn build() {
        set_Widget(
            gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(0)
                .css_classes(["widget", "workspaces"])
                .build(),
        );

        let buttons: [gtk4::Button; SIZE] = std::array::from_fn(|idx| {
            gtk4::Button::builder()
                .child(&gtk4::Label::new(Some(&format!("{}", idx + 1))))
                .build()
        });
        for button in &buttons {
            Widget().append(button);
        }
        set_Buttons(buttons);
    }
}

pub(crate) mod htop {
    use super::*;

    widget!(Widget, gtk4::Button);

    pub(crate) fn build() {
        set_Widget(
            gtk4::Button::builder()
                .css_classes(["widget", "terminal", "padded", "clickable"])
                .child(&gtk4::Label::new(Some("Htop")))
                .build(),
        );
    }
}

pub(crate) mod weather {
    use super::*;

    widget!(Widget, gtk4::Button);
    widget!(Label, gtk4::Label);

    pub(crate) fn build() {
        set_Label(gtk4::Label::new(None));

        set_Widget(
            gtk4::Button::builder()
                .css_classes(["widget", "weather", "padded", "clickable"])
                .child(Label())
                .build(),
        );
    }
}

pub(crate) mod language {
    use super::*;

    widget!(Widget, gtk4::CenterBox);
    widget!(Label, gtk4::Label);

    pub(crate) fn build() {
        set_Label(gtk4::Label::new(None));
        set_Widget(
            gtk4::CenterBox::builder()
                .css_classes(["widget", "language", "padded"])
                .center_widget(Label())
                .build(),
        );
    }
}

pub(crate) mod sound {
    use super::*;

    widget!(Widget, gtk4::Box);
    widget!(Image, gtk4::Image);
    widget!(Scale, gtk4::Scale);

    pub(crate) fn build() {
        set_Widget(
            gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(0)
                .css_classes(["widget", "sound", "padded"])
                .build(),
        );

        set_Image(gtk4::Image::builder().icon_name("dialog-question").build());
        Widget().append(Image());

        set_Scale(
            gtk4::Scale::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .adjustment(&gtk4::Adjustment::builder().lower(0.0).upper(1.0).build())
                .css_classes(["sound-slider"])
                .build(),
        );
        Widget().append(Scale());
    }
}

pub(crate) mod cpu {
    use super::*;

    widget!(Widget, gtk4::Box);
    const CPUS: usize = 12;
    widget!(Labels, [gtk4::Label; CPUS]);

    pub(crate) fn build() {
        set_Widget(
            gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(3)
                .css_classes(["widget", "cpu", "padded"])
                .build(),
        );

        set_Labels(std::array::from_fn(|_| {
            gtk4::Label::builder().use_markup(true).build()
        }));

        for label in Labels().iter() {
            Widget().append(label);
        }
    }
}

pub(crate) mod ram {
    use super::*;

    widget!(Widget, gtk4::Button);
    widget!(Label, gtk4::Label);

    pub(crate) fn build() {
        set_Label(gtk4::Label::new(None));

        set_Widget(
            gtk4::Button::builder()
                .css_classes(["widget", "memory", "padded", "clickable"])
                .child(Label())
                .build(),
        );
    }
}

pub(crate) mod network {
    use super::*;
    use crate::icons::wifi_icon;

    widget!(Widget, gtk4::Button);
    widget!(Label, gtk4::Label);
    widget!(Image, gtk4::Image);

    pub(crate) fn build() {
        set_Label(gtk4::Label::new(None));
        set_Image(gtk4::Image::builder().gicon(wifi_icon()).build());

        let wrapper = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        wrapper.append(Label());
        wrapper.append(Image());

        set_Widget(
            gtk4::Button::builder()
                .css_classes(["widget", "network", "padded", "clickable"])
                .cursor(&gtk4::gdk::Cursor::builder().name("pointer").build())
                .child(&wrapper)
                .build(),
        );
    }
}

pub(crate) mod clock {
    use super::*;

    widget!(Widget, gtk4::CenterBox);
    widget!(Label, gtk4::Label);

    pub(crate) fn build() {
        set_Label(gtk4::Label::new(None));

        set_Widget(
            gtk4::CenterBox::builder()
                .css_classes(["widget", "clock", "padded"])
                .center_widget(Label())
                .build(),
        );
    }
}

pub(crate) mod session {
    use super::*;
    use crate::icons::power_icon;

    widget!(Widget, gtk4::Button);

    pub(crate) fn build() {
        set_Widget(
            gtk4::Button::builder()
                .css_classes(["widget", "power", "padded", "clickable"])
                .cursor(&gtk4::gdk::Cursor::builder().name("pointer").build())
                .child(&gtk4::Image::builder().gicon(power_icon()).build())
                .build(),
        );
    }
}

pub(crate) fn setup() {
    let window = gtk4::Window::new();
    window.set_widget_name("TopBarWindow");

    let layout = gtk4::CenterBox::new();
    layout.add_css_class("main-wrapper");
    window.set_child(Some(&layout));

    let left = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    layout.set_start_widget(Some(&left));

    let right = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    layout.set_end_widget(Some(&right));

    macro_rules! build_widget {
        ($widget:ident, $parent:ident) => {
            $widget::build();
            $parent.append($widget::Widget())
        };
    }

    // left
    build_widget!(workspaces, left);

    // right
    build_widget!(htop, right);
    build_widget!(weather, right);
    build_widget!(language, right);
    build_widget!(sound, right);
    build_widget!(cpu, right);
    build_widget!(ram, right);
    build_widget!(network, right);
    build_widget!(clock, right);
    build_widget!(session, right);

    set_TopBarWindow(window);
}

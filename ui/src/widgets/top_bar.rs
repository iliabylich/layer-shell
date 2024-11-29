use crate::widgets::widget;
use gtk4::prelude::{BoxExt, GtkWindowExt, WidgetExt};

widget!(TopBarWindow, gtk4::Window);

pub(crate) mod workspaces {
    use super::*;
    widget!(Widget, gtk4::Box);
    const SIZE: usize = 10;
    widget!(Buttons, [gtk4::Button; SIZE]);

    pub(crate) fn build() -> &'static gtk4::Box {
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

        Widget()
    }
}

pub(crate) mod htop {
    use super::*;
    widget!(Widget, gtk4::Button);

    pub(crate) fn build() -> &'static gtk4::Button {
        set_Widget(
            gtk4::Button::builder()
                .css_classes(["widget", "terminal", "padded", "clickable"])
                .child(&gtk4::Label::new(Some("Htop")))
                .build(),
        );

        Widget()
    }
}

pub(crate) mod weather {
    use super::*;
    widget!(Widget, gtk4::Button);
    widget!(Label, gtk4::Label);

    pub(crate) fn build() -> &'static gtk4::Button {
        set_Label(gtk4::Label::new(None));

        set_Widget(
            gtk4::Button::builder()
                .css_classes(["widget", "weather", "padded", "clickable"])
                .child(Label())
                .build(),
        );

        Widget()
    }
}

pub(crate) mod language {
    use super::*;
    widget!(Widget, gtk4::CenterBox);
    widget!(Label, gtk4::Label);

    pub(crate) fn build() -> &'static gtk4::CenterBox {
        set_Label(gtk4::Label::new(None));
        set_Widget(
            gtk4::CenterBox::builder()
                .css_classes(["widget", "language", "padded"])
                .center_widget(Label())
                .build(),
        );

        Widget()
    }
}

pub(crate) mod sound {
    use super::*;
    widget!(Widget, gtk4::Box);
    widget!(Image, gtk4::Image);
    widget!(Scale, gtk4::Scale);

    pub(crate) fn build() -> &'static gtk4::Box {
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

        Widget()
    }
}

pub(crate) mod cpu {
    use super::*;
    widget!(Widget, gtk4::Box);
    const CPUS: usize = 12;
    widget!(Labels, [gtk4::Label; CPUS]);

    pub(crate) fn build() -> &'static gtk4::Box {
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

        Widget()
    }
}

pub(crate) mod ram {
    use super::*;
    widget!(Widget, gtk4::Button);
    widget!(Label, gtk4::Label);

    pub(crate) fn build() -> &'static gtk4::Button {
        set_Label(gtk4::Label::new(None));

        set_Widget(
            gtk4::Button::builder()
                .css_classes(["widget", "memory", "padded", "clickable"])
                .child(Label())
                .build(),
        );

        Widget()
    }
}

pub(crate) mod network {
    use super::*;
    widget!(Widget, gtk4::Button);
    widget!(Label, gtk4::Label);
    widget!(Image, gtk4::Image);

    pub(crate) fn build() -> &'static gtk4::Button {
        set_Label(gtk4::Label::new(None));
        set_Image(gtk4::Image::new());

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

        Widget()
    }
}

pub(crate) mod clock {
    use super::*;
    widget!(Widget, gtk4::CenterBox);
    widget!(Label, gtk4::Label);

    pub(crate) fn build() -> &'static gtk4::CenterBox {
        set_Label(gtk4::Label::new(None));

        set_Widget(
            gtk4::CenterBox::builder()
                .css_classes(["widget", "clock", "padded"])
                .center_widget(Label())
                .build(),
        );

        Widget()
    }
}

pub(crate) mod session {
    use super::*;

    widget!(Widget, gtk4::Button);
    widget!(Image, gtk4::Image);

    pub(crate) fn build() -> &'static gtk4::Button {
        set_Image(gtk4::Image::new());

        set_Widget(
            gtk4::Button::builder()
                .css_classes(["widget", "power", "padded", "clickable"])
                .cursor(&gtk4::gdk::Cursor::builder().name("pointer").build())
                .child(Image())
                .build(),
        );

        Widget()
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

    // left
    left.append(workspaces::build());

    // right
    right.append(htop::build());
    right.append(weather::build());
    right.append(language::build());
    right.append(sound::build());
    right.append(cpu::build());
    right.append(ram::build());
    right.append(network::build());
    right.append(clock::build());
    right.append(session::build());

    set_TopBarWindow(window);
}

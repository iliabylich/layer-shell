// This code was autogenerated with `dbus-codegen-rust --client blocking -o src/dbus/gen/status_notifier_item.rs`, see https://github.com/diwic/dbus-rs
use dbus as dbus;
#[allow(unused_imports)]
use dbus::arg;
use dbus::blocking;

pub(crate) trait OrgKdeStatusNotifierItem {
    fn context_menu(&self, x_: i32, y_: i32) -> Result<(), dbus::Error>;
    fn activate(&self, x_: i32, y_: i32) -> Result<(), dbus::Error>;
    fn secondary_activate(&self, x_: i32, y_: i32) -> Result<(), dbus::Error>;
    fn scroll(&self, delta: i32, orientation: &str) -> Result<(), dbus::Error>;
    fn category(&self) -> Result<String, dbus::Error>;
    fn id(&self) -> Result<String, dbus::Error>;
    fn title(&self) -> Result<String, dbus::Error>;
    fn status(&self) -> Result<String, dbus::Error>;
    fn window_id(&self) -> Result<u32, dbus::Error>;
    fn icon_theme_path(&self) -> Result<String, dbus::Error>;
    fn icon_name(&self) -> Result<String, dbus::Error>;
    fn icon_pixmap(&self) -> Result<Vec<(i32, i32, Vec<u8>,)>, dbus::Error>;
    fn overlay_icon_name(&self) -> Result<String, dbus::Error>;
    fn overlay_icon_pixmap(&self) -> Result<Vec<(i32, i32, Vec<u8>,)>, dbus::Error>;
    fn attention_icon_name(&self) -> Result<String, dbus::Error>;
    fn attention_icon_pixmap(&self) -> Result<Vec<(i32, i32, Vec<u8>,)>, dbus::Error>;
    fn attention_movie_name(&self) -> Result<String, dbus::Error>;
    fn tool_tip(&self) -> Result<(String, Vec<(i32, i32, Vec<u8>,)>, String, String,), dbus::Error>;
    fn menu(&self) -> Result<dbus::Path<'static>, dbus::Error>;
    fn item_is_menu(&self) -> Result<bool, dbus::Error>;
}

#[derive(Debug)]
pub(crate) struct OrgKdeStatusNotifierItemNewTitle {
}

impl arg::AppendAll for OrgKdeStatusNotifierItemNewTitle {
    fn append(&self, _: &mut arg::IterAppend) {
    }
}

impl arg::ReadAll for OrgKdeStatusNotifierItemNewTitle {
    fn read(_: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgKdeStatusNotifierItemNewTitle {
        })
    }
}

impl dbus::message::SignalArgs for OrgKdeStatusNotifierItemNewTitle {
    const NAME: &'static str = "NewTitle";
    const INTERFACE: &'static str = "org.kde.StatusNotifierItem";
}

#[derive(Debug)]
pub(crate) struct OrgKdeStatusNotifierItemNewIcon {
}

impl arg::AppendAll for OrgKdeStatusNotifierItemNewIcon {
    fn append(&self, _: &mut arg::IterAppend) {
    }
}

impl arg::ReadAll for OrgKdeStatusNotifierItemNewIcon {
    fn read(_: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgKdeStatusNotifierItemNewIcon {
        })
    }
}

impl dbus::message::SignalArgs for OrgKdeStatusNotifierItemNewIcon {
    const NAME: &'static str = "NewIcon";
    const INTERFACE: &'static str = "org.kde.StatusNotifierItem";
}

#[derive(Debug)]
pub(crate) struct OrgKdeStatusNotifierItemNewAttentionIcon {
}

impl arg::AppendAll for OrgKdeStatusNotifierItemNewAttentionIcon {
    fn append(&self, _: &mut arg::IterAppend) {
    }
}

impl arg::ReadAll for OrgKdeStatusNotifierItemNewAttentionIcon {
    fn read(_: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgKdeStatusNotifierItemNewAttentionIcon {
        })
    }
}

impl dbus::message::SignalArgs for OrgKdeStatusNotifierItemNewAttentionIcon {
    const NAME: &'static str = "NewAttentionIcon";
    const INTERFACE: &'static str = "org.kde.StatusNotifierItem";
}

#[derive(Debug)]
pub(crate) struct OrgKdeStatusNotifierItemNewOverlayIcon {
}

impl arg::AppendAll for OrgKdeStatusNotifierItemNewOverlayIcon {
    fn append(&self, _: &mut arg::IterAppend) {
    }
}

impl arg::ReadAll for OrgKdeStatusNotifierItemNewOverlayIcon {
    fn read(_: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgKdeStatusNotifierItemNewOverlayIcon {
        })
    }
}

impl dbus::message::SignalArgs for OrgKdeStatusNotifierItemNewOverlayIcon {
    const NAME: &'static str = "NewOverlayIcon";
    const INTERFACE: &'static str = "org.kde.StatusNotifierItem";
}

#[derive(Debug)]
pub(crate) struct OrgKdeStatusNotifierItemNewToolTip {
}

impl arg::AppendAll for OrgKdeStatusNotifierItemNewToolTip {
    fn append(&self, _: &mut arg::IterAppend) {
    }
}

impl arg::ReadAll for OrgKdeStatusNotifierItemNewToolTip {
    fn read(_: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgKdeStatusNotifierItemNewToolTip {
        })
    }
}

impl dbus::message::SignalArgs for OrgKdeStatusNotifierItemNewToolTip {
    const NAME: &'static str = "NewToolTip";
    const INTERFACE: &'static str = "org.kde.StatusNotifierItem";
}

#[derive(Debug)]
pub(crate) struct OrgKdeStatusNotifierItemNewStatus {
    pub(crate) status: String,
}

impl arg::AppendAll for OrgKdeStatusNotifierItemNewStatus {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.status, i);
    }
}

impl arg::ReadAll for OrgKdeStatusNotifierItemNewStatus {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgKdeStatusNotifierItemNewStatus {
            status: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for OrgKdeStatusNotifierItemNewStatus {
    const NAME: &'static str = "NewStatus";
    const INTERFACE: &'static str = "org.kde.StatusNotifierItem";
}

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target=T>> OrgKdeStatusNotifierItem for blocking::Proxy<'a, C> {

    fn context_menu(&self, x_: i32, y_: i32) -> Result<(), dbus::Error> {
        self.method_call("org.kde.StatusNotifierItem", "ContextMenu", (x_, y_, ))
    }

    fn activate(&self, x_: i32, y_: i32) -> Result<(), dbus::Error> {
        self.method_call("org.kde.StatusNotifierItem", "Activate", (x_, y_, ))
    }

    fn secondary_activate(&self, x_: i32, y_: i32) -> Result<(), dbus::Error> {
        self.method_call("org.kde.StatusNotifierItem", "SecondaryActivate", (x_, y_, ))
    }

    fn scroll(&self, delta: i32, orientation: &str) -> Result<(), dbus::Error> {
        self.method_call("org.kde.StatusNotifierItem", "Scroll", (delta, orientation, ))
    }

    fn category(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "Category")
    }

    fn id(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "Id")
    }

    fn title(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "Title")
    }

    fn status(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "Status")
    }

    fn window_id(&self) -> Result<u32, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "WindowId")
    }

    fn icon_theme_path(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "IconThemePath")
    }

    fn icon_name(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "IconName")
    }

    fn icon_pixmap(&self) -> Result<Vec<(i32, i32, Vec<u8>,)>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "IconPixmap")
    }

    fn overlay_icon_name(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "OverlayIconName")
    }

    fn overlay_icon_pixmap(&self) -> Result<Vec<(i32, i32, Vec<u8>,)>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "OverlayIconPixmap")
    }

    fn attention_icon_name(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "AttentionIconName")
    }

    fn attention_icon_pixmap(&self) -> Result<Vec<(i32, i32, Vec<u8>,)>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "AttentionIconPixmap")
    }

    fn attention_movie_name(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "AttentionMovieName")
    }

    fn tool_tip(&self) -> Result<(String, Vec<(i32, i32, Vec<u8>,)>, String, String,), dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "ToolTip")
    }

    fn menu(&self) -> Result<dbus::Path<'static>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "Menu")
    }

    fn item_is_menu(&self) -> Result<bool, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(self, "org.kde.StatusNotifierItem", "ItemIsMenu")
    }
}

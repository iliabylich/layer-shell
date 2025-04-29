from gi.repository import Gdk, GdkPixbuf, Gio, GLib, Gtk
from liblayer_shell_io import Commands, TrayIcon
from utils.subscribe import subscribe


class Tray(Gtk.Box):
    def __init__(self, app, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.app = app
        self.max_icons_count = 10
        subscribe(self)

        self.set_orientation(Gtk.Orientation.HORIZONTAL)
        self.set_spacing(10)
        self.set_css_classes(["widget", "tray", "padded"])
        self.set_name("Tray")

    def cleanup(self):
        for child in self.get_children(self):
            for grandchild in self.get_children(child):
                grandchild.unparent()
            self.remove(child)

    def add(self, tray_app):
        tray_app = TrayApp(app=self.app, tray_app=tray_app)
        self.append(tray_app.icon)

    def on_tray(self, event):
        self.cleanup()

        for idx, app in enumerate(event.apps):
            if idx > self.max_icons_count:
                break
            self.add(app)

    def get_children(self, widget):
        out = []
        child = widget.get_first_child()
        while child is not None:
            out.append(child)
            child = child.get_next_sibling()
        return out


class TrayApp:
    def __init__(self, app, tray_app):
        self.icon = TrayAppIcon(tray_app.icon)

        action_group = Gio.SimpleActionGroup.new()
        menu = TrayMenu(
            app=app,
            tray_item=tray_app.root_item,
            action_group=action_group,
        )

        self.popover_menu = Gtk.PopoverMenu.new_from_model(menu)
        self.popover_menu.set_has_arrow(False)
        self.popover_menu.set_parent(self.icon)

        gesture = Gtk.GestureClick.new()
        gesture.connect("pressed", self.on_click)
        self.icon.add_controller(gesture)
        self.icon.insert_action_group("tray", action_group)

    def on_click(self, gesture, n_press, x, y):
        self.popover_menu.popup()


class TrayAppIcon(Gtk.Image):
    def __init__(self, tray_icon, *args, **kwargs):
        super().__init__(*args, **kwargs)

        if isinstance(tray_icon, TrayIcon.Path):
            self.set_from_file(tray_icon.path)
        elif isinstance(tray_icon, TrayIcon.Name):
            self.set_from_icon_name(tray_icon.name)
        elif isinstance(tray_icon, TrayIcon.PixmapVariant):
            bytes = GLib.Bytes(tray_icon.bytes)
            pixbuf = GdkPixbuf.Pixbuf.new_from_bytes(
                bytes,
                GdkPixbuf.Colorspace.RGB,
                True,
                8,
                tray_icon.w,
                tray_icon.h,
                4 * tray_icon.w,
            )
            texture = Gdk.Texture.new_for_pixbuf(pixbuf)
            self.set_from_paintable(texture)
        elif isinstance(tray_icon, TrayIcon.Unset):
            self.set_from_gicon(self.tray_icons.question_mark)
        else:
            print(f"Unknown icon type {tray_icon}")


class TrayMenu(Gio.Menu):
    def __init__(self, app, tray_item, action_group, *args, **kwargs):
        super().__init__(*args, **kwargs)

        for idx, child in enumerate(tray_item.children):
            if not child.visible:
                continue

            menu_item = TrayMenuItemFor(
                app=app, tray_item=child, action_group=action_group, idx=idx
            )

            self.append_item(menu_item)


def TrayMenuItemFor(app, tray_item, action_group, idx):
    if tray_item.children_display == "submenu":
        cls = TrayNestedMenuItem
    else:
        if tray_item.enabled:
            if tray_item.toggle_type == "checkmark":
                cls = TrayCheckboxItem
            elif tray_item.toggle_type == "radio":
                cls = TrayRadioItem
            else:
                cls = TrayRegularItem
        else:
            cls = TrayDisabledItem

    return cls(app=app, tray_item=tray_item, action_group=action_group, idx=idx)


class BaseTrayMenuItem(Gio.MenuItem):
    def __init__(self, app, tray_item, action_group, idx, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.app = app
        self.tray_item = tray_item
        self.action_group = action_group
        self.idx = idx

        self.set_label(tray_item.label)

    def uuid(self):
        return self.tray_item.uuid

    def children_display(self):
        return self.tray_item.children_display

    def toggle_type(self):
        return self.tray_item.toggle_type

    def action_name(self):
        return f"{self.idx}"

    def on_activate(self, action, parameter):
        Commands.trigger_tray(self.app.ui_ctx, self.uuid())


class TrayNestedMenuItem(BaseTrayMenuItem):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        submenu = TrayMenu(
            app=self.app, tray_item=self.tray_item, action_group=self.action_group
        )
        self.set_submenu(submenu)


class TrayCheckboxItem(BaseTrayMenuItem):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        action = Gio.SimpleAction.new_stateful(
            self.action_name(),
            None,
            GLib.Variant("b", self.tray_item.toggle_state == 1),
        )
        action.connect("activate", self.on_activate)
        self.action_group.add_action(action)
        self.set_action_and_target_value(f"tray.{self.action_name()}", None)


class TrayRadioItem(BaseTrayMenuItem):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        action = Gio.SimpleAction.new_stateful(
            self.action_name(),
            GLib.VariantType("b"),
            GLib.Variant("b", self.tray_item.toggle_state == 1),
        )
        action.connect("activate", self.on_activate)
        self.action_group.add_action(action)
        self.set_action_and_target_value(
            f"tray.{self.action_name()}", GLib.Variant("b", True)
        )


class TrayRegularItem(BaseTrayMenuItem):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        action = Gio.SimpleAction.new(self.action_name())
        action.connect("activate", self.on_activate)
        self.action_group.add_action(action)
        self.set_action_and_target_value(f"tray.{self.action_name()}", None)


class TrayDisabledItem(BaseTrayMenuItem):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self.set_action_and_target_value("tray.noop", None)

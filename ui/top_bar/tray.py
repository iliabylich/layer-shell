from gi.repository import Gtk, Gdk, Gio, GLib, GdkPixbuf
from utils.subscribe import subscribe
from liblayer_shell_io import TrayIcon, Commands


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
        for child in get_children(self):
            for grandchild in get_children(child):
                grandchild.unparent()
            self.remove(child)

    def add(self, tray_app):
        icon = Gtk.Image()

        if isinstance(tray_app.icon, TrayIcon.Path):
            icon.set_from_file(tray_app.icon.path)
        elif isinstance(tray_app.icon, TrayIcon.Name):
            icon.set_from_icon_name(tray_app.icon.name)
        elif isinstance(tray_app.icon, TrayIcon.PixmapVariant):
            bytes = GLib.Bytes(tray_app.icon.bytes)
            pixbuf = GdkPixbuf.Pixbuf.new_from_bytes(
                bytes,
                GdkPixbuf.Colorspace.RGB,
                True,
                8,
                tray_app.icon.w,
                tray_app.icon.h,
                4 * tray_app.icon.w,
            )
            texture = Gdk.Texture.new_for_pixbuf(pixbuf)
            icon.set_from_paintable(texture)
        elif isinstance(tray_app.icon, TrayIcon.Unset):
            icon.set_from_gicon(self.tray_app.icons.question_mark)
        else:
            print(f"Unknown icon type {tray_app.icon}")

        action_group = Gio.SimpleActionGroup.new()
        menu = self.make_menu(tray_app.root_item, action_group)

        popover_menu = Gtk.PopoverMenu.new_from_model(menu)
        popover_menu.set_has_arrow(False)
        popover_menu.set_parent(icon)

        gesture = Gtk.GestureClick.new()
        gesture.connect(
            "pressed",
            lambda gesture,
            n_press,
            x,
            y,
            popover_menu=popover_menu: popover_menu.popup(),
        )
        icon.add_controller(gesture)

        self.append(icon)

        icon.insert_action_group("tray", action_group)

    def on_tray(self, event):
        self.cleanup()

        for i in range(0, self.max_icons_count):
            if i < len(event.apps):
                self.add(event.apps[i])

    def make_menu(self, tray_item, action_group):
        menu = Gio.Menu.new()

        for i in range(0, len(tray_item.children)):
            child = tray_item.children[i]
            if not child.visible:
                continue

            menu_item = Gio.MenuItem.new(child.label, None)

            uuid = child.uuid
            children_display = child.children_display
            toggle_type = child.toggle_type
            action_name = f"{i}"

            cb = lambda action, parameter, uuid=uuid: self.trigger_tray_item(uuid)

            if children_display == "submenu":
                # nested menu
                submenu = self.make_menu(child, action_group)
                menu_item.set_submenu(submenu)
            else:
                # element
                if child.enabled:
                    if toggle_type == "checkmark":
                        # checkbox
                        action = Gio.SimpleAction.new_stateful(
                            action_name,
                            GLib.VariantType("b"),
                            GLib.Variant("b", child.toggle_state == 1),
                        )
                        action.connect("activate", cb)
                        action_group.add_action(action)
                        menu_item.set_action_and_target_value(
                            f"tray.{action_name}", None
                        )
                    elif toggle_type == "radio":
                        # radio
                        action = Gio.SimpleAction.new_stateful(
                            action_name,
                            GLib.VariantType("b"),
                            GLib.Variant("b", child.toggle_state == 1),
                        )
                        action.connect("activate", cb)
                        action_group.add_action(action)
                        menu_item.set_action_and_target_value(
                            f"tray.{action_name}", GLib.Variant("b", True)
                        )
                    else:
                        action = Gio.SimpleAction.new(action_name)
                        action.connect("activate", cb)
                        action_group.add_action(action)
                        menu_item.set_action_and_target_value(
                            f"tray.{action_name}", None
                        )
                else:
                    menu_item.set_action_and_target_value("tray.noop", None)

            menu.append_item(menu_item)

        return menu

    def trigger_tray_item(self, uuid):
        Commands.trigger_tray(self.app.ui_ctx, uuid)


def get_children(widget):
    out = []
    child = widget.get_first_child()
    while child is not None:
        out.append(child)
        child = child.get_next_sibling()
    return out

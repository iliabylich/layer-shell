from gi.repository import Gtk, Pango
from liblayer_shell_io import LauncherAppIcon


class Row(Gtk.Box):
    def __init__(self):
        super().__init__()

        self.set_orientation(Gtk.Orientation.HORIZONTAL)
        self.set_spacing(0)
        self.set_css_classes(["row"])

        self.image = Gtk.Image()
        self.image.set_icon_size(Gtk.IconSize.LARGE)

        self.label = Gtk.Label.new("...")
        self.label.set_xalign(0.0)
        self.label.set_valign(Gtk.Align.CENTER)
        self.label.set_ellipsize(Pango.EllipsizeMode.END)

        self.append(self.image)
        self.append(self.label)

    def update(self, launcher_app):
        self.show()
        if launcher_app.selected:
            self.add_css_class("active")
        else:
            self.remove_css_class("active")

        if isinstance(launcher_app.icon, LauncherAppIcon.IconName):
            self.image.set_from_icon_name(launcher_app.icon._0)
        elif isinstance(launcher_app.icon, LauncherAppIcon.IconPath):
            self.image.set_from_file(launcher_app.icon._0)
        else:
            print(f"Unknown icon type {launcher_app.icon}")
        self.label.set_label(launcher_app.name)

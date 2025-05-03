from gi.repository import Gdk, Gtk
from liblayer_shell_io import Commands


class ChangeTheme(Gtk.Button):
    def __init__(self, app, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.app = app
        self.app.pub_sub.subscribe(self)

        self.set_css_classes(["widget", "power", "padded", "clickable"])
        self.set_cursor(Gdk.Cursor.new_from_name("pointer"))
        self.set_name("ChangeTheme")

        image = Gtk.Image.new_from_gicon(self.app.icons.change_theme)
        self.set_child(image)

        self.connect("clicked", self.on_click)

    def on_click(self, _):
        Commands.change_theme(self.app.ui_ctx)

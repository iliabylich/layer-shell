from gi.repository import Gdk, Gtk
from utils.commands import Commands
from utils.context import ctx


class ChangeTheme(Gtk.Button):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        ctx.pub_sub.subscribe(self)

        self.set_css_classes(["widget", "power", "padded", "clickable"])
        self.set_cursor(Gdk.Cursor.new_from_name("pointer"))
        self.set_name("ChangeTheme")

        image = Gtk.Image.new_from_gicon(ctx.icons.change_theme)
        self.set_child(image)

        self.connect("clicked", self.on_click)

    def on_click(self, _):
        Commands.change_theme()

from gi.repository import Gdk, Gtk
from utils.context import ctx


class Power(Gtk.Button):
    def __init__(self):
        super().__init__()

        self.set_css_classes(["widget", "power", "padded", "clickable"])
        self.set_cursor(Gdk.Cursor.new_from_name("pointer"))
        self.set_name("Power")

        image = Gtk.Image.new_from_gicon(ctx.icons.power)
        self.set_child(image)

        self.connect("clicked", self.on_click)

    def on_click(self, _):
        ctx.windows.session.toggle()

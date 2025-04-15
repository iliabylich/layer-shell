from gi.repository import Gtk, Gdk


class Power(Gtk.Button):
    def __init__(self, app, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.app = app

        self.set_css_classes(["widget", "power", "padded", "clickable"])
        self.set_cursor(Gdk.Cursor.new_from_name("pointer"))
        self.set_name("Power")

        image = Gtk.Image.new_from_gicon(self.app.icons.power)
        self.set_child(image)

        self.connect("clicked", self.on_click)

    def on_click(self, _):
        self.app.session.toggle()

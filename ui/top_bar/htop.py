from gi.repository import Gtk
from utils.subscribe import subscribe


class Htop(Gtk.Button):
    def __init__(self, app, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.app = app
        subscribe(self)

        self.set_label("HTop")
        self.set_css_classes(["widget", "terminal", "padded", "clickable"])
        self.set_name("HTop")
        self.connect("clicked", self.on_click)

    def on_click(self, _):
        self.app.htop.toggle()

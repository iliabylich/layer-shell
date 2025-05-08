from gi.repository import Gtk
from utils.context import ctx


class Htop(Gtk.Button):
    def __init__(self):
        super().__init__()
        ctx.pub_sub.subscribe(self)

        self.set_label("HTop")
        self.set_css_classes(["widget", "terminal", "padded", "clickable"])
        self.set_name("HTop")
        self.connect("clicked", self.on_click)

    def on_click(self, _):
        ctx.windows.htop.toggle()

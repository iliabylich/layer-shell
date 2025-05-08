from gi.repository import Gtk
from utils.context import ctx


class Clock(Gtk.Label):
    def __init__(self):
        super().__init__()
        ctx.pub_sub.subscribe(self)

        self.set_css_classes(["widget", "clock", "padded"])
        self.set_name("Clock")

    def on_time(self, event):
        self.set_label(event.time)

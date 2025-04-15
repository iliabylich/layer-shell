from gi.repository import Gtk
from utils.subscribe import subscribe


class Clock(Gtk.Label):
    def __init__(self, app, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.app = app
        subscribe(self)

        self.set_css_classes(["widget", "clock", "padded"])
        self.set_name("Clock")

    def on_time(self, event):
        self.set_label(event.time)

from gi.repository import Gtk
from liblayer_shell_io import Commands


class Memory(Gtk.Button):
    def __init__(self, app, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.app = app
        self.app.pub_sub.subscribe(self)

        self.set_label("--")
        self.set_css_classes(["widget", "memory", "padded", "clickable"])
        self.set_name("Memory")
        self.connect("clicked", self.on_click)

    def on_memory(self, event):
        used = round(event.used, 1)
        total = round(event.total, 1)
        self.set_label(f"RAM {used}G/{total}G")

    def on_click(self, _):
        Commands.spawn_system_monitor(self.app.ui_ctx)

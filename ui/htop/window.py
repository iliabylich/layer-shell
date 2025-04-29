from gi.repository import Gtk4LayerShell
from utils.vte_window import VteWindow


class Window(VteWindow):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, command=["htop"], **kwargs)

        self.set_name("HtopWindow")
        self.set_size_request(1000, 700)

        Gtk4LayerShell.init_for_window(self)
        Gtk4LayerShell.set_layer(self, Gtk4LayerShell.Layer.OVERLAY)
        Gtk4LayerShell.set_namespace(self, "LayerShell/Htop")
        Gtk4LayerShell.set_keyboard_mode(self, Gtk4LayerShell.KeyboardMode.EXCLUSIVE)

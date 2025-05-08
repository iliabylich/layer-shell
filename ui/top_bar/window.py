from gi.repository import Gtk, Gtk4LayerShell
from top_bar.change_theme import ChangeTheme
from top_bar.clock import Clock
from top_bar.cpu import CPU
from top_bar.htop import Htop
from top_bar.language import Language
from top_bar.memory import Memory
from top_bar.network import Network
from top_bar.power import Power
from top_bar.sound import Sound
from top_bar.tray import Tray
from top_bar.weather import Weather
from top_bar.workspaces import Workspaces
from utils.base_window import BaseWindow


class Window(BaseWindow):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self.set_name("TopBarWindow")
        self.set_css_classes(["top-bar-window"])

        layout = Gtk.CenterBox()
        layout.set_css_classes(["wrapper"])
        self.set_child(layout)

        left = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        right = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=4)

        left.append(Workspaces())
        left.append(ChangeTheme())

        right.append(Tray())
        right.append(Weather())
        right.append(Htop())
        right.append(Language())
        right.append(Sound())
        right.append(CPU())
        right.append(Memory())
        right.append(Network())
        right.append(Clock())
        right.append(Power())

        layout.set_start_widget(left)
        layout.set_end_widget(right)

        Gtk4LayerShell.init_for_window(self)
        Gtk4LayerShell.set_layer(self, Gtk4LayerShell.Layer.TOP)
        Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.TOP, True)
        Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.LEFT, True)
        Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.RIGHT, True)
        Gtk4LayerShell.set_namespace(self, "LayerShell/TopBar")
        Gtk4LayerShell.auto_exclusive_zone_enable(self)

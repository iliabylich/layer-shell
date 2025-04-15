from gi.repository import Gtk, Gtk4LayerShell
from utils.base_window import BaseWindow
from top_bar.workspaces import Workspaces
from top_bar.change_theme import ChangeTheme
from top_bar.power import Power
from top_bar.clock import Clock
from top_bar.network import Network
from top_bar.htop import Htop
from top_bar.memory import Memory
from top_bar.cpu import CPU
from top_bar.sound import Sound
from top_bar.language import Language
from top_bar.weather import Weather
from top_bar.tray import Tray


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

        left.append(Workspaces(app=self.get_application()))
        left.append(ChangeTheme(app=self.get_application()))

        right.append(Tray(app=self.get_application()))
        right.append(Weather(app=self.get_application()))
        right.append(Htop(app=self.get_application()))
        right.append(Language(app=self.get_application()))
        right.append(Sound(app=self.get_application()))
        right.append(CPU(app=self.get_application()))
        right.append(Memory(app=self.get_application()))
        right.append(Network(app=self.get_application()))
        right.append(Clock(app=self.get_application()))
        right.append(Power(app=self.get_application()))

        layout.set_start_widget(left)
        layout.set_end_widget(right)

        Gtk4LayerShell.init_for_window(self)
        Gtk4LayerShell.set_layer(self, Gtk4LayerShell.Layer.TOP)
        Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.TOP, True)
        Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.LEFT, True)
        Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.RIGHT, True)
        Gtk4LayerShell.set_margin(self, Gtk4LayerShell.Edge.TOP, 0)
        Gtk4LayerShell.set_namespace(self, "LayerShell/TopBar")

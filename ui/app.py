import os

from gi.repository import GLib, Gtk
from htop.window import Window as Htop
from icons.icons import Icons
from launcher.window import Window as Launcher
from liblayer_shell_io import init, poll_events, spawn_thread
from ping.window import Window as Ping
from top_bar.window import Window as TopBar
from utils.css_loader import CssLoader
from weather.window import Window as Weather
from windows.session import Session


class App(Gtk.Application):
    def __init__(self, **kwargs):
        super().__init__(**kwargs)

        self.root_path = os.path.dirname(os.path.abspath(__file__))
        self.io_ctx, self.ui_ctx = init()
        self.connect("startup", self.on_startup)
        self.connect("activate", self.on_activate)

    def on_startup(self, _):
        self.css_loader = CssLoader(self)
        self.css_loader.load()

    def on_activate(self, _):
        self.icons = Icons(self)

        self.top_bar = TopBar(application=self)
        self.session = Session(application=self)
        self.htop = Htop(application=self)
        self.weather = Weather(application=self)
        self.launcher = Launcher(application=self)
        self.ping = Ping(application=self)

        GLib.timeout_add(50, self.on_tick)

        print("Finished bulding widgets...")
        spawn_thread(self.io_ctx)

        self.top_bar.present()

    def on_tick(self):
        poll_events(self.ui_ctx)
        return True

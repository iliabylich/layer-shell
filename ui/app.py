from gi.repository import GLib, Gtk
from htop.window import Window as Htop
from icons.icons import Icons
from launcher.window import Window as Launcher
from liblayer_shell_io import IO
from ping.window import Window as Ping
from session.window import Window as Session
from top_bar.window import Window as TopBar
from utils.context import ctx
from utils.css_loader import CssLoader
from utils.pub_sub import PubSub
from weather.window import Window as Weather


class App(Gtk.Application):
    def __init__(self):
        super().__init__(application_id="org.me.LayerShell")

        ctx.app = self
        ctx.io_ctx, ctx.ui_ctx = IO.init()
        ctx.pub_sub = PubSub()
        self.connect("startup", self.on_startup)
        self.connect("activate", self.on_activate)

    def on_startup(self, _):
        self.css_loader = CssLoader()
        self.css_loader.load()

    def on_activate(self, _):
        ctx.icons = Icons()

        ctx.windows.top_bar = TopBar(application=self)
        ctx.windows.session = Session(application=self)
        ctx.windows.htop = Htop(application=self)
        ctx.windows.weather = Weather(application=self)
        ctx.windows.launcher = Launcher(application=self)
        ctx.windows.ping = Ping(application=self)

        GLib.timeout_add(50, self.on_tick)

        print("Finished bulding widgets...")
        IO.spawn_thread(ctx.io_ctx)

        ctx.windows.top_bar.present()

    def on_tick(self):
        ctx.pub_sub.poll_events()
        return True

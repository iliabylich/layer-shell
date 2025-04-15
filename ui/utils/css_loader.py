from os import environ
from gi.repository import Gtk, Gdk
from utils.subscribe import subscribe
from liblayer_shell_io import main_css


class CssLoader:
    def __init__(self, app, **kwargs):
        super().__init__(**kwargs)
        self.app = app
        subscribe(self)

    def load(self):
        full_css = self.theme_css() + "\n" + main_css()
        self.provider = Gtk.CssProvider()
        self.provider.connect("parsing-error", self.on_error)
        self.provider.load_from_string(full_css)

        display = Gdk.Display.get_default()
        Gtk.StyleContext.add_provider_for_display(
            display, self.provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )
        print("Finished loading CSS...")

    def theme_css(self):
        home = environ["HOME"]
        path = f"{home}/.config/layer-shell/theme.css"
        try:
            with open(path, "r", encoding="utf-8") as f:
                return f.read()
        except FileNotFoundError:
            return ""

    def on_error(self, provider, section, error):
        print("Failed to parse CSS:", section.to_string(), error)

    def on_reload_styles(self, event):
        print("Reloading styles...")
        display = Gdk.Display.get_default()
        Gtk.StyleContext.remove_provider_for_display(display, self.provider)
        self.load()

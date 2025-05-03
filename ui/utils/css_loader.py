import importlib.resources
import os

from gi.repository import Gdk, Gtk


class CssLoader:
    def __init__(self, app, **kwargs):
        super().__init__(**kwargs)
        self.app = app
        self.app.pub_sub.subscribe(self)

    def load(self):
        full_css = self.theme_css() + "\n" + self.main_css()
        self.provider = Gtk.CssProvider()
        self.provider.connect("parsing-error", self.on_error)
        self.provider.load_from_string(full_css)

        display = Gdk.Display.get_default()
        Gtk.StyleContext.add_provider_for_display(
            display, self.provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )
        print("Finished loading CSS...")

    def theme_css(self):
        home = os.environ["HOME"]
        path = f"{home}/.config/layer-shell/theme.css"
        try:
            with open(path, encoding="utf-8") as f:
                return f.read()
        except FileNotFoundError:
            return ""

    def main_css(self):
        return importlib.resources.read_text(__package__, "main.css")

    def on_error(self, provider, section, error):
        print("Failed to parse CSS:", section.to_string(), error)

    def on_reload_styles(self, event):
        print("Reloading styles...")
        display = Gdk.Display.get_default()
        Gtk.StyleContext.remove_provider_for_display(display, self.provider)
        self.load()

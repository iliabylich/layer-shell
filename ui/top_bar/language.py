from gi.repository import Gtk
from utils.subscribe import subscribe


class Language(Gtk.Label):
    def __init__(self, app, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.app = app
        subscribe(self)

        self.set_label("--")
        self.set_css_classes(["widget", "language", "padded"])
        self.set_name("Language")

    def on_language(self, event):
        lang = event.lang
        if lang == "English (US)":
            lang = "EN"
        elif lang == "Polish":
            lang = "PL"
        else:
            lang = "??"
        self.set_label(lang)

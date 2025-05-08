import importlib.resources
import os

from gi.repository import Gio, GLib


class Icons:
    def __init__(self):
        for entry in importlib.resources.files(__package__).iterdir():
            if entry.is_file() and entry.name.lower().endswith(".png"):
                self.declare_icon(entry)

    def declare_icon(self, entry):
        name = os.path.splitext(entry.name)[0]
        try:
            with entry.open("rb") as f:
                bytes = GLib.Bytes(f.read())
                setattr(self, name, Gio.BytesIcon.new(bytes))
        except FileNotFoundError:
            return None

from gi.repository import Gio, GLib
from liblayer_shell_io import Icons as LibIcons


class Icons:
    def __init__(self):
        for icon_name in LibIcons.names():
            self.icon(icon_name)

    def icon(self, icon_name):
        method = getattr(LibIcons, icon_name, None)
        if method is None:
            print(f"icon {icon_name} doesn't exist")
            return None
        bytes = GLib.Bytes(method())
        setattr(self, icon_name, Gio.BytesIcon.new(bytes))

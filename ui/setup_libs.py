from ctypes import CDLL
import gi
from os import environ, path
import sys

if "DEV" in environ:
    debug_dir = path.normpath(
        path.join(path.dirname(__file__), "..", "target", "debug")
    )
    print(f"Development mode, adding {debug_dir} to PATH")
    sys.path.append(debug_dir)
else:
    release_dir = "/lib/x86_64-linux-gnu"
    print(f"Release mode, adding {release_dir} to PATH")
    sys.path.append(release_dir)

gi.require_version("Gtk", "4.0")
gi.require_version("Gtk4LayerShell", "1.0")
gi.require_version("GLib", "2.0")
gi.require_version("Gdk", "4.0")
gi.require_version("GdkPixbuf", "2.0")
gi.require_version("Gio", "2.0")
gi.require_version("Pango", "1.0")
gi.require_version("Vte", "3.91")

CDLL("libgtk4-layer-shell.so")

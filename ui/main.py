#!/usr/bin/env python3

from setproctitle import setproctitle
from ctypes import CDLL
import gi
import os
import sys

CDLL("libgtk4-layer-shell.so")

gi.require_version("Gtk", "4.0")
gi.require_version("Gtk4LayerShell", "1.0")
gi.require_version("GLib", "2.0")
gi.require_version("Gdk", "4.0")
gi.require_version("GdkPixbuf", "2.0")
gi.require_version("Gio", "2.0")
gi.require_version("Pango", "1.0")
gi.require_version("Vte", "3.91")


if "DEV" in os.environ:
    debug_dir = os.path.normpath(
        os.path.join(os.path.dirname(__file__), "..", "target", "debug")
    )
    print(f"Development mode, adding {debug_dir} to PATH")
    sys.path.append(debug_dir)
else:
    release_dir = "/lib/x86_64-linux-gnu"
    print(f"Release mode, adding {release_dir} to PATH")
    sys.path.append(release_dir)

from app import App  # noqa: E402

setproctitle("layer-shell")

app = App(application_id="org.me.LayerShell")
app.run(None)

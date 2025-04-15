from os import environ
from gi.repository import Vte, GLib

from utils.base_window import BaseWindow


class VteWindow(BaseWindow):
    def __init__(self, command, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self.set_css_classes(["terminal-window"])
        self.toggle_on_escape()

        home = environ["HOME"]
        terminal = Vte.Terminal()
        terminal.spawn_async(
            Vte.PtyFlags.DEFAULT,  # pty_flags
            home,  # working_directory
            command,  # argv
            None,  # envv
            GLib.SpawnFlags.DEFAULT,  # spawn_flags
            None,  # child_setup
            None,  # child_setup_data + child_setup_data_destroy
            -1,  # timeout
            None,  # cancellable
            None,  # callback
            None,
        )  # user_data
        self.set_child(terminal)

from gi.repository import Gtk, Gtk4LayerShell
from utils.base_window import BaseWindow
from utils.commands import Commands
from utils.context import ctx


class Window(BaseWindow):
    def __init__(self, application):
        super().__init__(application=application)
        ctx.pub_sub.subscribe(self)

        self.set_name("SessionWindow")
        self.set_css_classes(["session-window"])
        self.toggle_on_escape()

        layout = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=200)
        layout.set_homogeneous(True)
        layout.set_css_classes(["wrapper"])
        self.set_child(layout)

        lock = Gtk.Button(label="Lock")
        lock.connect("clicked", self.on_lock_clicked)
        layout.append(lock)

        reboot = Gtk.Button(label="Reboot")
        reboot.connect("clicked", self.on_reboot_clicked)
        layout.append(reboot)

        shutdown = Gtk.Button(label="Shutdown")
        shutdown.connect("clicked", self.on_shutdown_clicked)
        layout.append(shutdown)

        logout = Gtk.Button(label="Logout")
        logout.connect("clicked", self.on_logout_clicked)
        layout.append(logout)

        Gtk4LayerShell.init_for_window(self)
        Gtk4LayerShell.set_layer(self, Gtk4LayerShell.Layer.OVERLAY)
        Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.TOP, True)
        Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.RIGHT, True)
        Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.BOTTOM, True)
        Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.LEFT, True)
        Gtk4LayerShell.set_namespace(self, "LayerShell/SessionScreen")
        Gtk4LayerShell.set_keyboard_mode(self, Gtk4LayerShell.KeyboardMode.EXCLUSIVE)

    def on_lock_clicked(self, _):
        self.toggle()
        Commands.lock()

    def on_reboot_clicked(self, _):
        self.toggle()
        Commands.reboot()

    def on_shutdown_clicked(self, _):
        self.toggle()
        Commands.shutdown()

    def on_logout_clicked(self, _):
        self.toggle()
        Commands.logout()

    def on_toggle_session_screen(self, _):
        self.toggle()

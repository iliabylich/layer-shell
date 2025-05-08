from gi.repository import Gdk, Gtk, Gtk4LayerShell
from launcher.row import Row
from utils.base_window import BaseWindow
from utils.commands import Commands
from utils.context import ctx


class Window(BaseWindow):
    def __init__(self, application):
        super().__init__(application=application)
        ctx.pub_sub.subscribe(self)

        self.set_name("LauncherWindow")
        self.set_size_request(700, -1)
        self.set_css_classes(["launcher-window"])

        layout = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        layout.set_css_classes(["wrapper"])
        self.set_child(layout)

        self.input = Gtk.SearchEntry()
        self.input.set_css_classes(["search-box"])
        self.input.set_hexpand(True)
        layout.append(self.input)

        scroll = Gtk.ScrolledWindow()
        scroll.set_css_classes(["scroll-list"])
        scroll.set_can_focus(False)
        layout.append(scroll)

        content = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        scroll.set_child(content)

        self.rows_count = 5
        self.rows = []
        for row in range(0, self.rows_count):
            row = Row()
            self.rows.append(row)
            content.append(row)

        Gtk4LayerShell.init_for_window(self)
        Gtk4LayerShell.set_layer(self, Gtk4LayerShell.Layer.OVERLAY)
        Gtk4LayerShell.set_namespace(self, "LayerShell/Launcher")
        Gtk4LayerShell.set_keyboard_mode(self, Gtk4LayerShell.KeyboardMode.EXCLUSIVE)

        self.input.connect("activate", self.on_submit)
        self.input.connect("changed", self.on_input_changed)

        ctrl = Gtk.EventControllerKey()
        ctrl.connect("key_pressed", self.on_key_pressed, False)
        ctrl.set_propagation_phase(Gtk.PropagationPhase.CAPTURE)
        self.add_controller(ctrl)

    def on_submit(self, _):
        Commands.launcher_exec_selected()
        self.toggle_and_reset()

    def on_input_changed(self, _):
        search = self.input.get_text()
        Commands.launcher_set_search(search)

    def on_key_pressed(self, ctrl, keyval, keycode, state, r):
        key = Gdk.keyval_name(keyval)
        if key == "Escape":
            self.toggle_and_reset()
        elif key == "Up":
            Commands.launcher_go_up()
        elif key == "Down":
            Commands.launcher_go_down()

        return False

    def toggle_and_reset(self):
        if self.get_visible():
            self.set_visible(False)
        else:
            self.send_launcher_reset_command()
            self.input.set_text("")
            self.set_visible(True)

    def on_toggle_launcher(self, _):
        self.toggle_and_reset()

    def on_launcher(self, event):
        for i in range(0, self.rows_count):
            row = self.rows[i]
            if i < len(event.apps):
                row.update(event.apps[i])
            else:
                row.hide()

    def send_launcher_reset_command(self):
        Commands.launcher_reset()
        return False

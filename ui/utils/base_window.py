from gi.repository import Gdk, Gtk


class BaseWindow(Gtk.Window):
    def toggle_on_escape(self):
        ctrl = Gtk.EventControllerKey()
        ctrl.connect("key_pressed", self.on_key_pressed)
        ctrl.set_propagation_phase(Gtk.PropagationPhase.CAPTURE)
        self.add_controller(ctrl)

    def on_key_pressed(self, ctrl, keyval, keycode, state):
        if Gdk.keyval_name(keyval) == "Escape":
            self.toggle()
            return True
        else:
            return False

    def toggle(self):
        self.set_visible(not self.get_visible())

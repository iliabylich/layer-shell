from gi.repository import Gtk


class TemperatureLabel(Gtk.Label):
    def __init__(self):
        super().__init__()
        self.set_label("??")

    def set_temperature(self, temperature):
        self.set_label(f"{temperature:>5.1f}℃")

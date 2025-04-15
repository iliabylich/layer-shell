from gi.repository import Gtk


class TemperatureLabel(Gtk.Label):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.set_label("??")

    def set_temperature(self, temperature):
        self.set_label(f"{temperature:>5.1f}℃")

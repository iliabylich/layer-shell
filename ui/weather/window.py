from gi.repository import Gtk, Gtk4LayerShell
from utils.base_window import BaseWindow
from utils.context import ctx
from weather.daily_grid import DailyGrid
from weather.hourly_grid import HourlyGrid


class Window(BaseWindow):
    def __init__(self, application):
        super().__init__(application=application)
        ctx.pub_sub.subscribe(self)

        self.set_name("WeatherWindow")
        self.set_css_classes(["weather-window"])
        self.toggle_on_escape()

        self.hourly_grid = HourlyGrid()
        self.daily_grid = DailyGrid()

        layout = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=50)
        self.set_child(layout)

        left_side = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        layout.append(left_side)
        left_side.append(Gtk.Label.new("Hourly"))
        left_side.append(self.hourly_grid)

        right_side = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        layout.append(right_side)
        right_side.append(Gtk.Label.new("Daily"))
        right_side.append(self.daily_grid)

        Gtk4LayerShell.init_for_window(self)
        Gtk4LayerShell.set_layer(self, Gtk4LayerShell.Layer.OVERLAY)
        Gtk4LayerShell.set_namespace(self, "LayerShell/Weather")
        Gtk4LayerShell.set_keyboard_mode(self, Gtk4LayerShell.KeyboardMode.EXCLUSIVE)

    def on_forecast_weather(self, event):
        self.hourly_grid.update(event.hourly)
        self.daily_grid.update(event.daily)

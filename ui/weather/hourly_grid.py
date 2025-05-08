from gi.repository import Gtk
from weather.base_grid import BaseGrid
from weather.temperature_icon import TemperatureIcon
from weather.temperature_label import TemperatureLabel


class HourlyGrid(BaseGrid):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, cols_count=3, rows_count=10, **kwargs)

        for row in range(0, self.rows_count):
            hour = Gtk.Label.new("??")
            self.attach(hour, 0, row, 1, 1)

            weather = TemperatureLabel()
            self.attach(weather, 1, row, 1, 1)

            image = TemperatureIcon()
            self.attach(image, 2, row, 1, 1)

    def update_row(self, weather_on_hour, row):
        self.get_child_at(0, row).set_label(weather_on_hour.hour)
        self.get_child_at(1, row).set_temperature(weather_on_hour.temperature)
        self.get_child_at(2, row).set_code(weather_on_hour.code)

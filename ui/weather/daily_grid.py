from gi.repository import Gtk
from weather.base_grid import BaseGrid
from weather.temperature_icon import TemperatureIcon
from weather.temperature_label import TemperatureLabel


class DailyGrid(BaseGrid):
    def __init__(self):
        super().__init__(cols_count=4, rows_count=6)

        for row in range(0, self.rows_count):
            day = Gtk.Label.new("??")
            self.attach(day, 0, row, 1, 1)

            min_weather = TemperatureLabel()
            self.attach(min_weather, 1, row, 1, 1)

            max_weather = TemperatureLabel()
            self.attach(max_weather, 2, row, 1, 1)

            image = TemperatureIcon()
            self.attach(image, 3, row, 1, 1)

    def update_row(self, weather_on_day, row):
        self.get_child_at(0, row).set_label(weather_on_day.day)
        self.get_child_at(1, row).set_temperature(weather_on_day.temperature_min)
        self.get_child_at(2, row).set_temperature(weather_on_day.temperature_max)
        self.get_child_at(3, row).set_code(weather_on_day.code)

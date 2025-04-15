from gi.repository import Gtk
from utils.weather_helper import WeatherHelper


class TemperatureIcon(Gtk.Image):
    def __init__(self, app, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.app = app

        self.set_from_gicon(self.app.icons.question_mark)

    def set_code(self, code):
        self.set_from_gicon(WeatherHelper.code_to_icon(code, self.app.icons))
        self.set_tooltip_text(WeatherHelper.code_to_description(code))

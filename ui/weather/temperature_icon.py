from gi.repository import Gtk
from utils.context import ctx
from utils.weather_helper import WeatherHelper


class TemperatureIcon(Gtk.Image):
    def __init__(self):
        super().__init__()

        self.set_from_gicon(ctx.icons.question_mark)

    def set_code(self, code):
        self.set_from_gicon(WeatherHelper.code_to_icon(code))
        self.set_tooltip_text(WeatherHelper.code_to_description(code))

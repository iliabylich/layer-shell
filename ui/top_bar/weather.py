from gi.repository import Gtk
from utils.context import ctx
from utils.weather_helper import WeatherHelper


class Weather(Gtk.Button):
    def __init__(self):
        super().__init__()
        ctx.pub_sub.subscribe(self)

        self.set_label("--")
        self.set_css_classes(["widget", "weather", "padded", "clickable"])
        self.set_name("Weather")
        self.connect("clicked", self.on_click)

    def on_current_weather(self, event):
        temperature = round(event.temperature, 1)
        description = WeatherHelper.code_to_description(event.code)
        self.set_label(f"{temperature}℃ {description}")

    def on_click(self, _):
        ctx.windows.weather.toggle()

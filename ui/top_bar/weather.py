from gi.repository import Gtk
from utils.weather_helper import WeatherHelper


class Weather(Gtk.Button):
    def __init__(self, app, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.app = app
        self.app.pub_sub.subscribe(self)

        self.set_label("--")
        self.set_css_classes(["widget", "weather", "padded", "clickable"])
        self.set_name("Weather")
        self.connect("clicked", self.on_click)

    def on_current_weather(self, event):
        temperature = round(event.temperature, 1)
        description = WeatherHelper.code_to_description(event.code)
        self.set_label(f"{temperature}℃ {description}")

    def on_click(self, _):
        self.app.weather.toggle()

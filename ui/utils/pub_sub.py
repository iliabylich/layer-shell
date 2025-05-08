from liblayer_shell_io import Event
from liblayer_shell_io import poll_events as io_poll_events
from utils.context import ctx

MAP = {
    Event.Memory: "memory",
    Event.CpuUsage: "cpu_usage",
    Event.Time: "time",
    Event.Workspaces: "workspaces",
    Event.Language: "language",
    Event.Launcher: "launcher",
    Event.Volume: "volume",
    Event.CurrentWeather: "current_weather",
    Event.ForecastWeather: "forecast_weather",
    Event.WifiStatus: "wifi_status",
    Event.NetworkSpeed: "network_speed",
    Event.NetworkList: "network_list",
    Event.Tray: "tray",
    Event.ToggleLauncher: "toggle_launcher",
    Event.ToggleSessionScreen: "toggle_session_screen",
    Event.ReloadStyles: "reload_styles",
}


class PubSub:
    def __init__(self):
        self.subscribers = []

    def poll_events(self):
        for event in io_poll_events(ctx.ui_ctx):
            self.publish(event)

    def subscribe(self, obj):
        self.subscribers.append(Subscriber(obj))

    def publish(self, event):
        for sub in self.subscribers:
            sub.on_event(event)


class Subscriber:
    def __init__(self, obj):
        self.obj = obj

    def on_event(self, event):
        event_handler = None
        for cls, method in MAP.items():
            if isinstance(event, cls):
                event_handler = f"on_{method}"
        if event_handler is None:
            print(f"Unknown event {event}, skipping")
            return
        method = getattr(self.obj, event_handler, None)
        if callable(method):
            method(event)

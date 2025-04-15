from liblayer_shell_io import Event, subscribe as io_subscribe

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


def subscribe(obj):
    io_subscribe(obj.app.ui_ctx, Subscriber(obj))


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

#include "include/utils/subscriber.hpp"

namespace utils {

Subscriber::Subscriber(void *ctx) : ctx(ctx) {
  layer_shell_io::layer_shell_io_subscribe(Subscriber::handle_event, this, ctx);
}

void Subscriber::handle_event(const layer_shell_io::Event *event, void *data) {
  ((Subscriber *)data)->on_io_event(event);
}

void Subscriber::on_io_event(const layer_shell_io::Event *event) {
  switch (event->tag) {
  case layer_shell_io::Event::Tag::Memory:
    on_memory_event(event->memory);
    break;
  case layer_shell_io::Event::Tag::CpuUsage:
    on_cpu_usage_event(event->cpu_usage);
    break;
  case layer_shell_io::Event::Tag::Time:
    on_time_event(event->time);
    break;
  case layer_shell_io::Event::Tag::Workspaces:
    on_workspaces_event(event->workspaces);
    break;
  case layer_shell_io::Event::Tag::Language:
    on_language_event(event->language);
    break;
  case layer_shell_io::Event::Tag::AppList:
    on_app_list_event(event->app_list);
    break;
  case layer_shell_io::Event::Tag::Volume:
    on_volume_event(event->volume);
    break;
  case layer_shell_io::Event::Tag::Mute:
    on_mute_event(event->mute);
    break;
  case layer_shell_io::Event::Tag::CurrentWeather:
    on_current_weather_event(event->current_weather);
    break;
  case layer_shell_io::Event::Tag::ForecastWeather:
    on_forecast_weather_event(event->forecast_weather);
    break;
  case layer_shell_io::Event::Tag::WifiStatus:
    on_wifi_status_event(event->wifi_status);
    break;
  case layer_shell_io::Event::Tag::NetworkSpeed:
    on_network_speed_event(event->network_speed);
    break;
  case layer_shell_io::Event::Tag::NetworkList:
    on_network_list_event(event->network_list);
    break;
  case layer_shell_io::Event::Tag::Tray:
    on_tray_event(event->tray);
    break;
  case layer_shell_io::Event::Tag::ToggleLauncher:
    on_toggle_launcher_event();
    break;
  case layer_shell_io::Event::Tag::ToggleSessionScreen:
    on_toggle_session_screen_event();
    break;
  }
}

void on_memory_event(layer_shell_io::Event::Memory_Body) {}
void on_cpu_usage_event(layer_shell_io::Event::CpuUsage_Body) {}
void on_time_event(layer_shell_io::Event::Time_Body) {}
void on_workspaces_event(layer_shell_io::Event::Workspaces_Body) {}
void on_language_event(layer_shell_io::Event::Language_Body) {}
void on_app_list_event(layer_shell_io::Event::AppList_Body) {}
void on_volume_event(layer_shell_io::Event::Volume_Body) {}
void on_mute_event(layer_shell_io::Event::Mute_Body) {}
void on_current_weather_event(layer_shell_io::Event::CurrentWeather_Body) {}
void on_forecast_weather_event(layer_shell_io::Event::ForecastWeather_Body) {}
void on_wifi_status_event(layer_shell_io::Event::WifiStatus_Body) {}
void on_network_speed_event(layer_shell_io::Event::NetworkSpeed_Body) {}
void on_network_list_event(layer_shell_io::Event::NetworkList_Body) {}
void on_tray_event(layer_shell_io::Event::Tray_Body) {}
void on_toggle_launcher_event() {}
void on_toggle_session_screen_event() {}

} // namespace utils

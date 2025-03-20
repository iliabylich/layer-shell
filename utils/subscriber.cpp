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
    on_io_event(event->memory);
    break;
  case layer_shell_io::Event::Tag::CpuUsage:
    on_io_event(event->cpu_usage);
    break;
  case layer_shell_io::Event::Tag::Time:
    on_io_event(event->time);
    break;
  case layer_shell_io::Event::Tag::Workspaces:
    on_io_event(event->workspaces);
    break;
  case layer_shell_io::Event::Tag::Language:
    on_io_event(event->language);
    break;
  case layer_shell_io::Event::Tag::Launcher:
    on_io_event(event->launcher);
    break;
  case layer_shell_io::Event::Tag::Volume:
    on_io_event(event->volume);
    break;
  case layer_shell_io::Event::Tag::Mute:
    on_io_event(event->mute);
    break;
  case layer_shell_io::Event::Tag::CurrentWeather:
    on_io_event(event->current_weather);
    break;
  case layer_shell_io::Event::Tag::ForecastWeather:
    on_io_event(event->forecast_weather);
    break;
  case layer_shell_io::Event::Tag::WifiStatus:
    on_io_event(event->wifi_status);
    break;
  case layer_shell_io::Event::Tag::NetworkSpeed:
    on_io_event(event->network_speed);
    break;
  case layer_shell_io::Event::Tag::NetworkList:
    on_io_event(event->network_list);
    break;
  case layer_shell_io::Event::Tag::Tray:
    on_io_event(event->tray);
    break;
  case layer_shell_io::Event::Tag::ToggleLauncher:
    on_toggle_launcher_event();
    break;
  case layer_shell_io::Event::Tag::ToggleSessionScreen:
    on_toggle_session_screen_event();
    break;
  case layer_shell_io::Event::Tag::ReloadStyles:
    on_reload_styles();
    break;
  }
}

void Subscriber::on_io_event(layer_shell_io::Event::Memory_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::CpuUsage_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::Time_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::Workspaces_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::Language_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::Launcher_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::Volume_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::Mute_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::CurrentWeather_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::ForecastWeather_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::WifiStatus_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::NetworkSpeed_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::NetworkList_Body) {}
void Subscriber::on_io_event(layer_shell_io::Event::Tray_Body) {}
void Subscriber::on_toggle_launcher_event() {}
void Subscriber::on_toggle_session_screen_event() {}
void Subscriber::on_reload_styles() {}

} // namespace utils

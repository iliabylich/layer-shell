#include "include/utils/subscriber.hpp"
#include "bindings.hpp"

namespace utils {

Subscriber::Subscriber(io::Subscriptions *subs) {
  io::io_subscription_list_add(subs, Subscriber::handle_event, this);
}

void Subscriber::handle_event(const io::Event *event, void *data) {
  ((Subscriber *)data)->on_io_event(event);
}

void Subscriber::on_io_event(const io::Event *event) {
  switch (event->tag) {
  case io::Event::Tag::Memory:
    on_io_event(event->memory);
    break;
  case io::Event::Tag::CpuUsage:
    on_io_event(event->cpu_usage);
    break;
  case io::Event::Tag::Time:
    on_io_event(event->time);
    break;
  case io::Event::Tag::Workspaces:
    on_io_event(event->workspaces);
    break;
  case io::Event::Tag::Language:
    on_io_event(event->language);
    break;
  case io::Event::Tag::Launcher:
    on_io_event(event->launcher);
    break;
  case io::Event::Tag::Volume:
    on_io_event(event->volume);
    break;
  case io::Event::Tag::Mute:
    on_io_event(event->mute);
    break;
  case io::Event::Tag::CurrentWeather:
    on_io_event(event->current_weather);
    break;
  case io::Event::Tag::ForecastWeather:
    on_io_event(event->forecast_weather);
    break;
  case io::Event::Tag::WifiStatus:
    on_io_event(event->wifi_status);
    break;
  case io::Event::Tag::NetworkSpeed:
    on_io_event(event->network_speed);
    break;
  case io::Event::Tag::NetworkList:
    on_io_event(event->network_list);
    break;
  case io::Event::Tag::Tray:
    on_io_event(event->tray);
    break;
  case io::Event::Tag::ToggleLauncher:
    on_toggle_launcher_event();
    break;
  case io::Event::Tag::ToggleSessionScreen:
    on_toggle_session_screen_event();
    break;
  case io::Event::Tag::ReloadStyles:
    on_reload_styles();
    break;
  }
}

void Subscriber::on_io_event(io::Event::Memory_Body) {}
void Subscriber::on_io_event(io::Event::CpuUsage_Body) {}
void Subscriber::on_io_event(io::Event::Time_Body) {}
void Subscriber::on_io_event(io::Event::Workspaces_Body) {}
void Subscriber::on_io_event(io::Event::Language_Body) {}
void Subscriber::on_io_event(io::Event::Launcher_Body) {}
void Subscriber::on_io_event(io::Event::Volume_Body) {}
void Subscriber::on_io_event(io::Event::Mute_Body) {}
void Subscriber::on_io_event(io::Event::CurrentWeather_Body) {}
void Subscriber::on_io_event(io::Event::ForecastWeather_Body) {}
void Subscriber::on_io_event(io::Event::WifiStatus_Body) {}
void Subscriber::on_io_event(io::Event::NetworkSpeed_Body) {}
void Subscriber::on_io_event(io::Event::NetworkList_Body) {}
void Subscriber::on_io_event(io::Event::Tray_Body) {}
void Subscriber::on_toggle_launcher_event() {}
void Subscriber::on_toggle_session_screen_event() {}
void Subscriber::on_reload_styles() {}

} // namespace utils

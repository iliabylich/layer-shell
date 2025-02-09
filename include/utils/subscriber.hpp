#pragma once

#include "bindings.hpp"

namespace utils {

class Subscriber {
public:
  Subscriber(void *ctx);

protected:
  void *ctx;

private:
  static void handle_event(const layer_shell_io::Event *event, void *data);

  void on_io_event(const layer_shell_io::Event *event);

  virtual void on_memory_event(layer_shell_io::Event::Memory_Body) {}
  virtual void on_cpu_usage_event(layer_shell_io::Event::CpuUsage_Body) {}
  virtual void on_time_event(layer_shell_io::Event::Time_Body) {}
  virtual void on_workspaces_event(layer_shell_io::Event::Workspaces_Body) {}
  virtual void on_language_event(layer_shell_io::Event::Language_Body) {}
  virtual void on_app_list_event(layer_shell_io::Event::AppList_Body) {}
  virtual void on_volume_event(layer_shell_io::Event::Volume_Body) {}
  virtual void on_mute_event(layer_shell_io::Event::Mute_Body) {}
  virtual void
  on_current_weather_event(layer_shell_io::Event::CurrentWeather_Body) {}
  virtual void
  on_forecast_weather_event(layer_shell_io::Event::ForecastWeather_Body) {}
  virtual void on_wifi_status_event(layer_shell_io::Event::WifiStatus_Body) {}
  virtual void
  on_network_speed_event(layer_shell_io::Event::NetworkSpeed_Body) {}
  virtual void on_network_list_event(layer_shell_io::Event::NetworkList_Body) {}
  virtual void on_tray_event(layer_shell_io::Event::Tray_Body) {}
  virtual void on_toggle_launcher_event() {}
  virtual void on_toggle_session_screen_event() {}
};

} // namespace utils

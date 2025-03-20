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

  virtual void on_io_event(layer_shell_io::Event::Memory_Body);
  virtual void on_io_event(layer_shell_io::Event::CpuUsage_Body);
  virtual void on_io_event(layer_shell_io::Event::Time_Body);
  virtual void on_io_event(layer_shell_io::Event::Workspaces_Body);
  virtual void on_io_event(layer_shell_io::Event::Language_Body);
  virtual void on_io_event(layer_shell_io::Event::Launcher_Body);
  virtual void on_io_event(layer_shell_io::Event::Volume_Body);
  virtual void on_io_event(layer_shell_io::Event::Mute_Body);
  virtual void on_io_event(layer_shell_io::Event::CurrentWeather_Body);
  virtual void on_io_event(layer_shell_io::Event::ForecastWeather_Body);
  virtual void on_io_event(layer_shell_io::Event::WifiStatus_Body);
  virtual void on_io_event(layer_shell_io::Event::NetworkSpeed_Body);
  virtual void on_io_event(layer_shell_io::Event::NetworkList_Body);
  virtual void on_io_event(layer_shell_io::Event::Tray_Body);
  virtual void on_toggle_launcher_event();
  virtual void on_toggle_session_screen_event();
  virtual void on_reload_styles();
};

} // namespace utils

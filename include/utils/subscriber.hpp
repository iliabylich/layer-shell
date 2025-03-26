#pragma once

#include "src/bindings.hpp"

namespace utils {

class Subscriber {
public:
  Subscriber(io::Ctx *ctx);

protected:
  io::Ctx *ctx;

private:
  static void handle_event(const io::Event *event, void *data);

  void on_io_event(const io::Event *event);

  virtual void on_io_event(io::Event::Memory_Body);
  virtual void on_io_event(io::Event::CpuUsage_Body);
  virtual void on_io_event(io::Event::Time_Body);
  virtual void on_io_event(io::Event::Workspaces_Body);
  virtual void on_io_event(io::Event::Language_Body);
  virtual void on_io_event(io::Event::Launcher_Body);
  virtual void on_io_event(io::Event::Volume_Body);
  virtual void on_io_event(io::Event::Mute_Body);
  virtual void on_io_event(io::Event::CurrentWeather_Body);
  virtual void on_io_event(io::Event::ForecastWeather_Body);
  virtual void on_io_event(io::Event::WifiStatus_Body);
  virtual void on_io_event(io::Event::NetworkSpeed_Body);
  virtual void on_io_event(io::Event::NetworkList_Body);
  virtual void on_io_event(io::Event::Tray_Body);
  virtual void on_toggle_launcher_event();
  virtual void on_toggle_session_screen_event();
  virtual void on_reload_styles();
};

} // namespace utils

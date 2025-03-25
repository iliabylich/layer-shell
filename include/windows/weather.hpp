#pragma once

#include "include/utils/subscriber.hpp"
#include "include/widgets/weather/daily_grid.hpp"
#include "include/widgets/weather/hourly_grid.hpp"
#include "include/windows/base.hpp"

namespace windows {

class Weather : public Base, utils::Subscriber {
public:
  Weather(const Glib::RefPtr<Gtk::Application> &app, void *ctx);
  void on_io_event(io::Event::ForecastWeather_Body data) override;
  static Weather *get();

private:
  widgets::weather::HourlyGrid hourly;
  widgets::weather::DailyGrid daily;
};

} // namespace windows

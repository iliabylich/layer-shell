#pragma once

#include "include/utils/subscriber.hpp"
#include "include/widgets/weather/daily_grid.hpp"
#include "include/widgets/weather/hourly_grid.hpp"
#include "include/windows/base.hpp"

namespace windows {

class Weather : public Base, utils::Subscriber {
public:
  static void init(const Glib::RefPtr<Gtk::Application> &app,
                   io::Subscriptions *subs);
  static Weather *get();
  void on_io_event(io::Event::ForecastWeather_Body data) override;

private:
  Weather(const Glib::RefPtr<Gtk::Application> &app, io::Subscriptions *subs);

  widgets::weather::HourlyGrid hourly;
  widgets::weather::DailyGrid daily;

  static Weather *instance;
};

} // namespace windows

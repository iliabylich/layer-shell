#pragma once

#include "include/utils/subscriber.hpp"
#include "include/windows/base.hpp"

namespace windows {

class Weather : public Base, utils::Subscriber {
public:
  Weather(const Glib::RefPtr<Gtk::Application> &app, void *ctx);
  void on_forecast_weather_event(
      layer_shell_io::Event::ForecastWeather_Body data) override;
  static Weather *get();

private:
  class Grid : public Gtk::Grid {
  public:
    Grid(size_t cols_count, size_t rows_count);
    size_t cols_count;
    size_t rows_count;
    Gtk::Label *label_at(size_t col, size_t row);
    Gtk::Image *image_at(size_t col, size_t row);
  };
  class HourlyGrid : public Grid {
  public:
    HourlyGrid();
    void update(layer_shell_io::WeatherOnHour weather, size_t row);
  };
  class DailyGrid : public Grid {
  public:
    DailyGrid();
    void update(layer_shell_io::WeatherOnDay weather, size_t row);
  };

  HourlyGrid hourly;
  DailyGrid daily;
};

} // namespace windows

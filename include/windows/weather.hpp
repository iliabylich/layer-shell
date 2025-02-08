#pragma once

#include "bindings.hpp"
#include "include/utils/subscription.hpp"
#include "include/utils/window-helper.hpp"
#include <gtkmm.h>

namespace windows {

class Weather : public Gtk::Window,
                public utils::Subscription<Weather>,
                public utils::WindowHelper<Weather> {
public:
  Weather();
  void activate(const Glib::RefPtr<Gtk::Application> &app, void *subscriptions);
  void on_io_event(const layer_shell_io::Event *event);

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

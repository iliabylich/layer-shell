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
  static int WIDTH;
  Weather();
  void activate(const Glib::RefPtr<Gtk::Application> &app);
  void on_io_event(const layer_shell_io::Event *event);

private:
  class Row : public Gtk::Box {
  public:
    Row();

  protected:
    Gtk::Label label;
    Gtk::Image image;
  };

  class HourlyRow : public Row {
  public:
    void update(layer_shell_io::WeatherOnHour weather);
  };

  class DailyRow : public Row {
  public:
    void update(layer_shell_io::WeatherOnDay weather);
  };

  std::vector<HourlyRow> hourly_rows;
  std::vector<DailyRow> daily_rows;
};

} // namespace windows

#include "include/windows/weather.hpp"
#include <gtk4-layer-shell.h>

namespace windows {

Weather::Weather(const Glib::RefPtr<Gtk::Application> &app,
                 io::Subscriptions *subs)
    : utils::Subscriber(subs) {
  set_name("WeatherWindow");
  set_css_classes({"weather-window"});
  set_application(app);

  Gtk::Box layout(Gtk::Orientation::HORIZONTAL, 50);

  {
    Gtk::Box side(Gtk::Orientation::VERTICAL, 0);
    Gtk::Label label("Hourly");
    side.append(label);
    side.append(hourly);
    layout.append(side);
  }

  {
    Gtk::Box side(Gtk::Orientation::VERTICAL, 0);
    Gtk::Label label("Daily");
    side.append(label);
    side.append(daily);
    layout.append(side);
  }

  set_child(layout);

  toggle_on_escape();

  auto win = gobj();
  gtk_layer_init_for_window(win);
  gtk_layer_set_layer(win, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(win, "LayerShell/Weather");
  gtk_layer_set_keyboard_mode(win, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);
}

void Weather::on_io_event(io::Event::ForecastWeather_Body data) {
  for (size_t row = 0; row < hourly.rows_count; row++) {
    auto weather = data.hourly.ptr[row];
    hourly.update(weather, row);
  }

  for (size_t row = 0; row < daily.rows_count; row++) {
    auto weather = data.daily.ptr[row];
    daily.update(weather, row);
  }
}

Weather *Weather::instance;
void Weather::init(const Glib::RefPtr<Gtk::Application> &app,
                   io::Subscriptions *subs) {
  instance = new Weather(app, subs);
}
Weather *Weather::get() { return instance; }

} // namespace windows

#include "include/windows/weather.hpp"
#include "include/utils/icons.hpp"
#include "include/utils/weather-helper.hpp"
#include <gtk4-layer-shell.h>

namespace windows {

Weather::Grid::Grid(size_t cols_count, size_t rows_count)
    : Gtk::Grid(), cols_count(cols_count), rows_count(rows_count) {
  for (size_t col = 0; col < cols_count; col++) {
    insert_column(col);
  }
  for (size_t row = 0; row < rows_count; row++) {
    insert_row(row);
  }
}

Gtk::Label *Weather::Grid::label_at(size_t col, size_t row) {
  return static_cast<Gtk::Label *>(get_child_at(col, row));
}
Gtk::Image *Weather::Grid::image_at(size_t col, size_t row) {
  return static_cast<Gtk::Image *>(get_child_at(col, row));
}

void update_image(Gtk::Image *image, layer_shell_io::WeatherCode code) {
  image->set(utils::WeatherHelper::weather_code_to_icon(code));
  image->set_tooltip_text(
      utils::WeatherHelper::weather_code_to_description(code));
}

std::string format_temperature(float temperature) {
  char buffer[100];
  sprintf(buffer, "%5.1f℃", temperature);
  return buffer;
}

// ----

Weather::HourlyGrid::HourlyGrid() : Weather::Grid(3, 10) {
  for (size_t row = 0; row < rows_count; row++) {
    Gtk::Label hour("??");
    // hour.set_xalign(0);
    attach(hour, 0, row);

    Gtk::Label weather("??");
    attach(weather, 1, row);

    Gtk::Image image;
    image.set(utils::Icons::question_mark_icon());
    attach(image, 2, row);
  }
}

void Weather::HourlyGrid::update(layer_shell_io::WeatherOnHour weather,
                                 size_t row) {
  label_at(0, row)->set_label(weather.hour);
  label_at(1, row)->set_label(format_temperature(weather.temperature));
  update_image(image_at(2, row), weather.code);
}

// ----

Weather::DailyGrid::DailyGrid() : Weather::Grid(4, 6) {
  for (size_t row = 0; row < rows_count; row++) {
    Gtk::Label day("??");
    attach(day, 0, row);

    Gtk::Label min_weather("??");
    attach(min_weather, 1, row);

    Gtk::Label max_weather("??");
    attach(max_weather, 2, row);

    Gtk::Image image;
    image.set(utils::Icons::question_mark_icon());
    attach(image, 3, row);
  }
}

void Weather::DailyGrid::update(layer_shell_io::WeatherOnDay weather,
                                size_t row) {
  label_at(0, row)->set_label(weather.day);

  label_at(1, row)->set_label(format_temperature(weather.temperature_min));
  label_at(2, row)->set_label(format_temperature(weather.temperature_max));
  update_image(image_at(3, row), weather.code);
}

#define DAILY_ROWS_COUNT 6
#define DAILY_COL_DAY 0
#define DAILY_COL_MIN_WEATHER 1
#define DAILY_COL_MAX_WEATHER 2
#define DAILY_COL_IMAGE 3
#define DAILY_COLS_COUNT 4

Weather::Weather(const Glib::RefPtr<Gtk::Application> &app, void *ctx)
    : utils::Subscriber(ctx) {
  set_name("WeatherWindow");
  set_css_classes({"widget-weather"});
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

void Weather::on_forecast_weather_event(
    layer_shell_io::Event::ForecastWeather_Body data) {
  for (size_t row = 0; row < hourly.rows_count; row++) {
    auto weather = data.hourly.ptr[row];
    hourly.update(weather, row);
  }

  for (size_t row = 0; row < daily.rows_count; row++) {
    auto weather = data.daily.ptr[row];
    daily.update(weather, row);
  }
}

} // namespace windows

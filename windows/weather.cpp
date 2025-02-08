#include "include/windows/weather.hpp"
#include "bindings.hpp"
#include "include/utils/weather-helper.hpp"

namespace windows {

Weather::Row::Row() : Gtk::Box() {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(0);

  label.set_label("...");
  image.set_pixel_size(24);
  append(label);
  append(image);
}

// ----

void Weather::HourlyRow::update(layer_shell_io::WeatherOnHour weather) {
  char buffer[100];
  sprintf(buffer, "%s' %5.1f℃", weather.hour, weather.temperature);
  label.set_label(buffer);
  label.set_tooltip_text(
      utils::WeatherHelper::weather_code_to_description(weather.code));

  image.set(utils::WeatherHelper::weather_code_to_icon(weather.code));
}

// ----

void Weather::DailyRow::update(layer_shell_io::WeatherOnDay weather) {
  char buffer[100];
  sprintf(buffer, "%s: %5.1f℃ - %5.1f℃", weather.day, weather.temperature_min,
          weather.temperature_max);
  label.set_label(buffer);
  label.set_tooltip_text(
      utils::WeatherHelper::weather_code_to_description(weather.code));

  image.set(utils::WeatherHelper::weather_code_to_icon(weather.code));
}

// ----

#define HOURLY_ROWS_COUNT 10
#define DAILY_ROWS_COUNT 6

Weather::Weather() : Gtk::Window() {
  set_name("WeatherWindow");
  set_css_classes({"widget-weather"});

  Gtk::Box layout(Gtk::Orientation::HORIZONTAL, 0);
  set_child(layout);
  {

    Gtk::Box list(Gtk::Orientation::VERTICAL, 0);
    list.add_css_class({"weather-left-side"});
    layout.append(list);

    Gtk::Label header("Hourly");
    list.append(header);
    for (size_t i = 0; i < HOURLY_ROWS_COUNT; i++) {
      HourlyRow row;
      list.append(row);
      hourly_rows.push_back(std::move(row));
    }
  }

  {
    Gtk::Box list(Gtk::Orientation::VERTICAL, 0);
    list.add_css_class({"weather-right-side"});
    layout.append(list);

    Gtk::Label header("Daily");
    list.append(header);
    for (size_t i = 0; i < DAILY_ROWS_COUNT; i++) {
      DailyRow row;
      list.append(row);
      daily_rows.push_back(std::move(row));
    }
  }
}

void Weather::activate(const Glib::RefPtr<Gtk::Application> &app) {
  set_application(app);
  toggle_on_escape();

  auto win = gobj();
  gtk_layer_init_for_window(win);
  gtk_layer_set_layer(win, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(win, "LayerShell/Weather");
  gtk_layer_set_keyboard_mode(win, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  subscribe_to_io_events();
}

void Weather::on_io_event(const layer_shell_io::Event *event) {
  if (event->tag == layer_shell_io::Event::Tag::ForecastWeather) {
    for (size_t i = 0; i < HOURLY_ROWS_COUNT; i++) {
      hourly_rows[i].update(event->forecast_weather.hourly.ptr[i]);
    }

    for (size_t i = 0; i < DAILY_ROWS_COUNT; i++) {
      daily_rows[i].update(event->forecast_weather.daily.ptr[i]);
    }
  }
}

} // namespace windows

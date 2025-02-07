#include "include/widgets/weather.hpp"
#include "include/utils/weather-helper.hpp"
#include "include/windows/top-bar.hpp"
#include "include/windows/weather.hpp"

namespace widgets {

Weather::Weather() : Gtk::Button() {
  set_css_classes({"widget", "weather", "padded", "clickable"});
  set_name("Weather");

  set_label("--");
}

void Weather::activate() {
  signal_clicked().connect([this]() {
    auto bottom_right = this->bottom_right_point(*windows::TopBar::instance());
    windows::Weather::move(bottom_right.get_x() - windows::Weather::WIDTH,
                           bottom_right.get_y());
    windows::Weather::toggle();
  });

  subscribe_to_io_events();
}

void Weather::on_io_event(const layer_shell_io::Event *event) {
  if (event->tag == layer_shell_io::Event::Tag::CurrentWeather) {
    char buffer[100];
    sprintf(buffer, "%.1f℃ %s", event->current_weather.temperature,
            utils::WeatherHelper::weather_code_to_description(
                event->current_weather.code));
    set_label(buffer);
  }
}

} // namespace widgets

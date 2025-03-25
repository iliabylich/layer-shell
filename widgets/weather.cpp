#include "include/widgets/weather.hpp"
#include "include/utils/weather-helper.hpp"
#include "include/windows/weather.hpp"

namespace widgets {

Weather::Weather(void *ctx) : Gtk::Button("--"), utils::Subscriber(ctx) {
  set_css_classes({"widget", "weather", "padded", "clickable"});
  set_name("Weather");
  signal_clicked().connect([]() { windows::Weather::get()->toggle(); });
}

void Weather::on_io_event(io::Event::CurrentWeather_Body data) {
  char buffer[100];
  sprintf(buffer, "%.1f℃ %s", data.temperature,
          utils::WeatherHelper::weather_code_to_description(data.code));
  set_label(buffer);
}

} // namespace widgets

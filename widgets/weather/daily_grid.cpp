#include "include/widgets/weather/daily_grid.hpp"
#include "include/utils/icons.hpp"

namespace widgets {
namespace weather {

DailyGrid::DailyGrid() : Grid(4, 6) {
  for (size_t row = 0; row < rows_count; row++) {
    Gtk::Label day("??");
    attach(day, 0, row);

    Gtk::Label min_weather("??");
    attach(min_weather, 1, row);

    Gtk::Label max_weather("??");
    attach(max_weather, 2, row);

    Gtk::Image image;
    image.set(utils::Icons::question_mark);
    attach(image, 3, row);
  }
}

void DailyGrid::update(io::WeatherOnDay weather, size_t row) {
  label_at(0, row)->set_label(weather.day);
  label_at(1, row)->set_label(format_temperature(weather.temperature_min));
  label_at(2, row)->set_label(format_temperature(weather.temperature_max));
  update_image(image_at(3, row), weather.code);
}

} // namespace weather
} // namespace widgets

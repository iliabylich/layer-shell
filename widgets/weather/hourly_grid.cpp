#include "include/widgets/weather/hourly_grid.hpp"
#include "include/utils/icons.hpp"

namespace widgets {
namespace weather {

HourlyGrid::HourlyGrid() : Grid(3, 10) {
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

void HourlyGrid::update(io::WeatherOnHour weather, size_t row) {
  label_at(0, row)->set_label(weather.hour);
  label_at(1, row)->set_label(format_temperature(weather.temperature));
  update_image(image_at(2, row), weather.code);
}

} // namespace weather
} // namespace widgets

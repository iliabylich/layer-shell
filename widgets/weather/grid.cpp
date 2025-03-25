#include "include/widgets/weather/grid.hpp"
#include "include/utils/weather-helper.hpp"

namespace widgets {
namespace weather {

Grid::Grid(size_t cols_count, size_t rows_count)
    : Gtk::Grid(), cols_count(cols_count), rows_count(rows_count) {
  for (size_t col = 0; col < cols_count; col++) {
    insert_column(col);
  }
  for (size_t row = 0; row < rows_count; row++) {
    insert_row(row);
  }
}

Gtk::Label *Grid::label_at(size_t col, size_t row) {
  return static_cast<Gtk::Label *>(get_child_at(col, row));
}
Gtk::Image *Grid::image_at(size_t col, size_t row) {
  return static_cast<Gtk::Image *>(get_child_at(col, row));
}

void Grid::update_image(Gtk::Image *image, io::WeatherCode code) {
  image->set(utils::WeatherHelper::weather_code_to_icon(code));
  image->set_tooltip_text(
      utils::WeatherHelper::weather_code_to_description(code));
}

std::string Grid::format_temperature(float temperature) {
  char buffer[100];
  sprintf(buffer, "%5.1f℃", temperature);
  return buffer;
}

} // namespace weather
} // namespace widgets

#pragma once

#include "bindings.hpp"
#include <gtkmm.h>

namespace widgets {
namespace weather {

class Grid : public Gtk::Grid {
public:
  Grid(size_t cols_count, size_t rows_count);
  size_t cols_count;
  size_t rows_count;
  Gtk::Label *label_at(size_t col, size_t row);
  Gtk::Image *image_at(size_t col, size_t row);

  static void update_image(Gtk::Image *image, io::WeatherCode code);
  static std::string format_temperature(float temperature);
};

} // namespace weather
} // namespace widgets

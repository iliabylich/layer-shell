#pragma once

#include "include/widgets/weather/grid.hpp"
#include <gtkmm.h>

namespace widgets {
namespace weather {

class HourlyGrid : public Grid {
public:
  HourlyGrid();
  void update(layer_shell_io::WeatherOnHour weather, size_t row);
};

} // namespace weather
} // namespace widgets

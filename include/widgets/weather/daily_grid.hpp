#pragma once

#include "include/widgets/weather/grid.hpp"
#include <gtkmm.h>

namespace widgets {
namespace weather {

class DailyGrid : public Grid {
public:
  DailyGrid();
  void update(layer_shell_io::WeatherOnDay weather, size_t row);
};

} // namespace weather
} // namespace widgets

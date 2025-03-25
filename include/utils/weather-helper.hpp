#pragma once

#include "bindings.hpp"
#include <gtkmm.h>

namespace utils {

class WeatherHelper {
public:
  static const char *weather_code_to_description(io::WeatherCode code);
  static Glib::RefPtr<const Gio::Icon> &
  weather_code_to_icon(io::WeatherCode code);
};

} // namespace utils

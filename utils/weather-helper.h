#ifndef WEATHER_HELPER_H
#define WEATHER_HELPER_H

#include "bindings.h"
#include <gio/gio.h>

const char *weather_code_to_description(IO_WeatherCode code);
GIcon *weather_code_to_icon(IO_WeatherCode code);

#endif // WEATHER_HELPER_H

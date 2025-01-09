#ifndef WEATHER_HELPER_H
#define WEATHER_HELPER_H

#include "bindings.h"
#include <gio/gio.h>

const char *weather_code_to_description(LAYER_SHELL_IO_WeatherCode code);
GIcon *weather_code_to_icon(LAYER_SHELL_IO_WeatherCode code);

#endif // WEATHER_HELPER_H

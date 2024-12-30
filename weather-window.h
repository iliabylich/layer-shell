#ifndef WEATHER_WINDOW_H
#define WEATHER_WINDOW_H

#include "gio/gio.h"

void init_weather_window(void);
void toggle_weather_window(void);
void activate_weather_window(GApplication *app);

#endif // WEATHER_WINDOW_H

#ifndef WEATHER_WINDOW_H
#define WEATHER_WINDOW_H

#include <gio/gio.h>
#include <stdint.h>

void init_weather_window(void);
void toggle_weather_window(void);
void activate_weather_window(GApplication *app);
void move_weather_window(uint32_t margin_left, uint32_t margin_top);
uint32_t weather_window_width(void);

#endif // WEATHER_WINDOW_H

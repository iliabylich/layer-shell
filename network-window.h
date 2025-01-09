#ifndef NETWORK_WINDOW_H
#define NETWORK_WINDOW_H

#include <gio/gio.h>
#include <stdint.h>

void init_network_window(void);
void toggle_network_window(void);
void activate_network_window(GApplication *app);
void move_network_window(uint32_t margin_left, uint32_t margin_top);
uint32_t network_window_width(void);

#endif // NETWORK_WINDOW_H

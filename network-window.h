#ifndef NETWORK_WINDOW_H
#define NETWORK_WINDOW_H

#include "gio/gio.h"

void init_network_window(void);
void toggle_network_window(void);
void activate_network_window(GApplication *app);

#endif // NETWORK_WINDOW_H

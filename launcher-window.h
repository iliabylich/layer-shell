#ifndef LAUNCHER_WINDOW_H
#define LAUNCHER_WINDOW_H

#include "gio/gio.h"

void init_launcher_window(void);
void toggle_launcher_window(void);
void activate_launcher_window(GApplication *app);

#endif // LAUNCHER_WINDOW_H

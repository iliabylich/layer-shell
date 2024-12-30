#ifndef HTOP_WINDOW_H
#define HTOP_WINDOW_H

#include "gio/gio.h"

void init_htop_window(void);
void toggle_htop_window(void);
void activate_htop_window(GApplication *app);

#endif // HTOP_WINDOW_H

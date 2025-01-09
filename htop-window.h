#ifndef HTOP_WINDOW_H
#define HTOP_WINDOW_H

#include <gio/gio.h>
#include <stdint.h>

void init_htop_window(void);
void toggle_htop_window(void);
void activate_htop_window(GApplication *app);
void move_htop_window(uint32_t margin_left, uint32_t margin_top);
uint32_t htop_window_width(void);

#endif // HTOP_WINDOW_H

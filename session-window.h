#ifndef SESSION_WINDOW_H
#define SESSION_WINDOW_H

#include <gio/gio.h>

void init_session_window(void);
void toggle_session_window(void);
void activate_session_window(GApplication *app);

#endif // SESSION_WINDOW_H

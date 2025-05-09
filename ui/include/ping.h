#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Ping, ping, PING, WINDOW, GtkWindow)

Ping *ping_new(GtkApplication *app);

#pragma once

#include "ui/include/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Ping, ping, PING, WINDOW, BaseWindow)

GtkWidget *ping_new(GtkApplication *app);

#define PING_TYPE ping_get_type()
#define PING(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, PING_TYPE, Ping)

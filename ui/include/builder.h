#pragma once

#include <gtk/gtk.h>

void init_builders();

#define DECLARE_BUILDER(name) GtkWidget *name##_get_widget(const char *name);

DECLARE_BUILDER(htop)
DECLARE_BUILDER(ping)
DECLARE_BUILDER(session)
DECLARE_BUILDER(top_bar)
DECLARE_BUILDER(weather)

#undef DECLARE_BUILDER

#pragma once

#include <gtk/gtk.h>

GtkWidget *clock_init();
void clock_refresh(GtkWidget *clock, const char *time);

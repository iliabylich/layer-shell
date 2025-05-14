#pragma once

#include <gtk/gtk.h>

#define Clock GtkLabel
#define CLOCK GTK_LABEL

GtkWidget *clock_new();
void clock_refresh(Clock *clock, const char *time);

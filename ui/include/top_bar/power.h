#pragma once

#include <gtk/gtk.h>

typedef void (*power_clicked_f)();

GtkWidget *power_init(power_clicked_f callback);

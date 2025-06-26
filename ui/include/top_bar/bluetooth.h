#pragma once

#include <gtk/gtk.h>

typedef void (*bluetooth_clicked_f)();

GtkWidget *bluetooth_init(bluetooth_clicked_f callback);

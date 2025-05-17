#pragma once

#include <gtk/gtk.h>

typedef void (*htop_button_clicked_f)();

GtkWidget *htop_button_init(htop_button_clicked_f callback);

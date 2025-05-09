#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(HtopButton, htop_button, HTOP_BUTTON, Widget, GtkButton)

GtkWidget *htop_button_new();

#define HTOP_BUTTON(obj)                                                       \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), htop_button_get_type(), HtopButton))

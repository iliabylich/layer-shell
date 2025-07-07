#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Bluetooth, bluetooth, BLUETOOTH, WIDGET, GtkWidget)

#define BLUETOOTH(obj)                                                         \
  G_TYPE_CHECK_INSTANCE_CAST(obj, bluetooth_get_type(), Bluetooth)

GtkWidget *bluetooth_new(void);

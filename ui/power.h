#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Power, power, POWER, WIDGET, GtkWidget)

#define POWER(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, power_get_type(), Power)

GtkWidget *power_new(void);

#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(HTop, htop, HTOP, WIDGET, GtkWidget)

#define HTOP(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, htop_get_type(), HTop)

GtkWidget *htop_new(void);

#pragma once

#include "ui/include/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Htop, htop, HTOP, WINDOW, BaseWindow)

GtkWidget *htop_new(GtkApplication *app);

#define HTOP_TYPE htop_get_type()
#define HTOP(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, HTOP_TYPE, Htop)

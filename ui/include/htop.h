#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Htop, htop, HTOP, WINDOW, GtkWindow)

Htop *htop_new(GtkApplication *app);

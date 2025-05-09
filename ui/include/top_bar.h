#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TopBar, top_bar, TOP_BAR, WINDOW, GtkWindow)

TopBar *top_bar_new(GtkApplication *app);
void top_bar_push_left(TopBar *top_bar, GtkWidget *child);
void top_bar_push_right(TopBar *top_bar, GtkWidget *child);

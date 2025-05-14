#pragma once

#include "ui/include/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TopBar, top_bar, TOP_BAR, WINDOW, BaseWindow)

GtkWidget *top_bar_new(GtkApplication *app);
void top_bar_push_left(TopBar *top_bar, GtkWidget *child);
void top_bar_push_right(TopBar *top_bar, GtkWidget *child);

#define TOP_BAR_TYPE top_bar_get_type()
#define TOP_BAR(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, TOP_BAR_TYPE, TopBar)

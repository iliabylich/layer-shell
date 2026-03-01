#pragma once

#include "ui/base_overlay.h"
#include "ui/view_models/io_model.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TopBar, top_bar, TOP_BAR, WINDOW, BaseOverlay)

#define TOP_BAR(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, top_bar_get_type(), TopBar)

GtkWidget *top_bar_new(GtkApplication *app, IOModel *model);
void top_bar_set_terminal_label(TopBar *self, const char *label);

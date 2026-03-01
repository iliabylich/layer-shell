#pragma once

#include "ui/base_overlay.h"
#include "ui/view_models/io_model.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TerminalOverlay, terminal_overlay, TERMINAL_OVERLAY,
                     OVERLAY, BaseOverlay)

#define TERMINAL_OVERLAY(obj)                                                  \
  G_TYPE_CHECK_INSTANCE_CAST(obj, terminal_overlay_get_type(), TerminalOverlay)

GtkWidget *terminal_overlay_new(GtkApplication *app, IOModel *model);

#pragma once

#include "ui/base_overlay.h"
#include "ui/view_models/io_model.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(PingOverlay, ping_overlay, PING_OVERLAY, OVERLAY,
                     BaseOverlay)

#define PING_OVERLAY(obj)                                                      \
  G_TYPE_CHECK_INSTANCE_CAST(obj, ping_overlay_get_type(), PingOverlay)

GtkWidget *ping_overlay_new(GtkApplication *app, IOModel *model);

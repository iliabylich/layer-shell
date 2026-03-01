#pragma once

#include "ui/base_overlay.h"
#include "ui/view_models/io_model.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(SessionOverlay, session_overlay, SESSION_OVERLAY, OVERLAY,
                     BaseOverlay)

#define SESSION_OVERLAY(obj)                                                   \
  G_TYPE_CHECK_INSTANCE_CAST(obj, session_overlay_get_type(), SessionOverlay)

GtkWidget *session_overlay_new(GtkApplication *app, IOModel *model);

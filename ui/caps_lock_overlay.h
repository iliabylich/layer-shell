#pragma once

#include "ui/base_overlay.h"
#include "ui/view_models/io_model.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(CapsLockOverlay, caps_lock_overlay, CAPS_LOCK_OVERLAY,
                     OVERLAY, BaseOverlay)

#define CAPS_LOCK_OVERLAY(obj)                                                 \
  G_TYPE_CHECK_INSTANCE_CAST(obj, caps_lock_overlay_get_type(), CapsLockOverlay)

GtkWidget *caps_lock_overlay_new(GtkApplication *app, IOModel *model);

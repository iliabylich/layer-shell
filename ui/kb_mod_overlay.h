#pragma once

#include "ui/base_overlay.h"
#include "ui/view_models/io_model.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(KbModOverlay, kb_mod_overlay, KB_MOD_OVERLAY, OVERLAY,
                     BaseOverlay)

#define KB_MOD_OVERLAY(obj)                                                    \
  G_TYPE_CHECK_INSTANCE_CAST(obj, kb_mod_overlay_get_type(), KbModOverlay)

GtkWidget *kb_mod_overlay_new(GtkApplication *app, IOModel *model);

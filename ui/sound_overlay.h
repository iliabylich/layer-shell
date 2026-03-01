#pragma once

#include "ui/base_overlay.h"
#include "ui/view_models/io_model.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(SoundOverlay, sound_overlay, SOUND_OVERLAY, OVERLAY,
                     BaseOverlay)

#define SOUND_OVERLAY(obj)                                                     \
  G_TYPE_CHECK_INSTANCE_CAST(obj, sound_overlay_get_type(), SoundOverlay)

GtkWidget *sound_overlay_new(GtkApplication *app, IOModel *model);

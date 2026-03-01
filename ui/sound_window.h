#pragma once

#include "ui/base_window.h"
#include "ui/sound_window_model.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(SoundWindow, sound_window, SOUND_WINDOW, WINDOW,
                     BaseWindow)

#define SOUND_WINDOW(obj)                                                      \
  G_TYPE_CHECK_INSTANCE_CAST(obj, sound_window_get_type(), SoundWindow)

GtkWidget *sound_window_new(GtkApplication *app,
                            SoundWindowModel *window_model);

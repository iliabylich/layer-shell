#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(SoundModel, sound_model, SOUND,
                     MODEL, GObject)

#define SOUND_MODEL(obj)                                                       \
  G_TYPE_CHECK_INSTANCE_CAST(obj, sound_model_get_type(), SoundModel)

SoundModel *sound_model_new(void);
void sound_model_set_initial(SoundModel *self, guint volume, gboolean muted);

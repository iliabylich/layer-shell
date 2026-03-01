#pragma once

#include "ui/window_model.h"

G_DECLARE_FINAL_TYPE(SoundWindowModel, sound_window_model, SOUND, WINDOW_MODEL,
                     WindowModel)

#define SOUND_WINDOW_MODEL(obj)                                                \
  G_TYPE_CHECK_INSTANCE_CAST(obj, sound_window_model_get_type(),               \
                             SoundWindowModel)

SoundWindowModel *sound_window_model_new(void);

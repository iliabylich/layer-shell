#include "ui/sound_window_model.h"

struct _SoundWindowModel {
  WindowModel parent_instance;
};

G_DEFINE_TYPE(SoundWindowModel, sound_window_model, window_model_get_type())

static void sound_window_model_init(SoundWindowModel *) {}
static void sound_window_model_class_init(SoundWindowModelClass *) {}

SoundWindowModel *sound_window_model_new(void) {
  return g_object_new(sound_window_model_get_type(), NULL);
}

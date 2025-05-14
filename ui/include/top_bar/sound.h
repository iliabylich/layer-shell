#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Sound, sound, SOUND, Widget, GtkBox)

GtkWidget *sound_new();
void sound_refresh(Sound *sound, uint32_t volume, bool muted);

#define SOUND_TYPE sound_get_type()
#define SOUND(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, SOUND_TYPE, Sound)

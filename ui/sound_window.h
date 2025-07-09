#pragma once

#include "bindings.h"
#include "ui/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(SoundWindow, sound_window, SOUND_WINDOW, WINDOW,
                     BaseWindow)

#define SOUND_WINDOW(obj)                                                      \
  G_TYPE_CHECK_INSTANCE_CAST(obj, sound_window_get_type(), SoundWindow)

GtkWidget *sound_window_new(GtkApplication *app);

void sound_window_set_initial_sound(SoundWindow *sound_window,
                                    IO_InitialSoundEvent event);
void sound_window_refresh_volume(SoundWindow *sound_window,
                                 IO_VolumeChangedEvent event);
void sound_window_refresh_mute(SoundWindow *sound_window,
                               IO_MuteChangedEvent event);

#pragma once

#include <gtk/gtk.h>

GtkWidget *sound_init(void);
void sound_refresh(GtkWidget *sound, uint32_t volume, bool muted);

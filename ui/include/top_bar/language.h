#pragma once

#include <gtk/gtk.h>

#define Language GtkLabel
#define LANGUAGE GTK_LABEL

GtkWidget *language_new();
void language_refresh(Language *language, const char *lang);

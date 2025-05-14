#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(ChangeTheme, change_theme, CHANGE_THEME, Widget, GtkButton)

GtkWidget *change_theme_new();

#define CHANGE_THEME_TYPE change_theme_get_type()
#define CHANGE_THEME(obj)                                                      \
  G_TYPE_CHECK_INSTANCE_CAST(obj, CHANGE_THEME_TYPE, ChangeTheme)

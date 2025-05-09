#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Language, language, LANGUAGE, Widget, GtkBox)

GtkWidget *language_new();
void language_refresh(Language *language, IO_CString lang);

#define LANGUAGE(obj)                                                          \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), language_get_type(), Language))

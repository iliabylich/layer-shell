#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Language, language, LANGUAGE, WIDGET, GtkWidget)

#define LANGUAGE(obj)                                                          \
  G_TYPE_CHECK_INSTANCE_CAST(obj, language_get_type(), Language)

GtkWidget *language_new(void);
void language_refresh(Language *language, IO_CString lang);

#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(LanguageModel, language_model, LANGUAGE, MODEL, GObject)

#define LANGUAGE_MODEL(obj)                                                    \
  G_TYPE_CHECK_INSTANCE_CAST(obj, language_model_get_type(), LanguageModel)

LanguageModel *language_model_new(void);

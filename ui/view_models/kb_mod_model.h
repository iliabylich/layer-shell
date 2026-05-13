#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(KbModModel, kb_mod_model, KB_MOD, MODEL, GObject)

#define KB_MOD_MODEL(obj)                                                      \
  G_TYPE_CHECK_INSTANCE_CAST(obj, kb_mod_model_get_type(), KbModModel)

KbModModel *kb_mod_model_new(void);
void kb_mod_model_update(KbModModel *self, IO_KbModKind kind, gboolean enabled);

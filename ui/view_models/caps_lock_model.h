#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(CapsLockModel, caps_lock_model, CAPS_LOCK,
                     MODEL, GObject)

#define CAPS_LOCK_MODEL(obj)                                                   \
  G_TYPE_CHECK_INSTANCE_CAST(obj, caps_lock_model_get_type(), CapsLockModel)

CapsLockModel *caps_lock_model_new(void);

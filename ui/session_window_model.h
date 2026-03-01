#pragma once

#include "ui/window_model.h"

G_DECLARE_FINAL_TYPE(SessionWindowModel, session_window_model, SESSION,
                     WINDOW_MODEL, WindowModel)

#define SESSION_WINDOW_MODEL(obj)                                              \
  G_TYPE_CHECK_INSTANCE_CAST(obj, session_window_model_get_type(),             \
                             SessionWindowModel)

SessionWindowModel *session_window_model_new(void);

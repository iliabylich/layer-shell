#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(WorkspacesModel, workspaces_model, WORKSPACES, MODEL,
                     GObject)

#define WORKSPACES_MODEL(obj)                                                  \
  G_TYPE_CHECK_INSTANCE_CAST(obj, workspaces_model_get_type(), WorkspacesModel)

WorkspacesModel *workspaces_model_new(void);

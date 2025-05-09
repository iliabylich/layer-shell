#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Workspaces, workspaces, WORKSPACES, Widget, GtkBox)

GtkWidget *workspaces_new();
void workspaces_emit_switched(Workspaces *workspaces, size_t idx);
void workspaces_refresh(Workspaces *workspaces, IO_CArray_usize ids,
                        size_t active_id);

#define WORKSPACES(obj)                                                        \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), workspaces_get_type(), Workspaces))

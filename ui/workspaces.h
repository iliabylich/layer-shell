#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Workspaces, workspaces_widget, WORKSPACES, WIDGET,
                     GtkWidget)

#define WORKSPACES(obj)                                                        \
  G_TYPE_CHECK_INSTANCE_CAST(obj, workspaces_widget_get_type(), Workspaces)

GtkWidget *workspaces_new(void);
void workspaces_refresh(Workspaces *workspaces,
                        struct IO_FFIArray_HyprlandWorkspace data);

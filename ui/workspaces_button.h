#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(WorkspacesButton, workspaces_button, WORKSPACES_BUTTON,
                     WIDGET, GtkWidget)

#define WORKSPACES_BUTTON(obj)                                                 \
  G_TYPE_CHECK_INSTANCE_CAST(obj, workspaces_button_get_type(),                \
                             WorkspacesButton)

GtkWidget *workspaces_button_new(guint num);

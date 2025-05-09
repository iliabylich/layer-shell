#pragma once

#include "ui/include/top_bar/workspaces.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(WorkspacesButton, workspaces_button, WORKSPACES_BUTTON,
                     Widget, GtkButton)

GtkWidget *workspaces_button_new(Workspaces *workspaces, size_t idx);
void workspaces_button_make_active(WorkspacesButton *self);
void workspaces_button_make_inactive(WorkspacesButton *self);

#define WORKSPACES_BUTTON(obj)                                                 \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), workspaces_button_get_type(),             \
                              WorkspacesButton))

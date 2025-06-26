#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

typedef void (*workspace_change_f)(size_t idx);

GtkWidget *workspaces_init(workspace_change_f callback);
void workspaces_emit_switched(GtkWidget *workspaces, size_t idx);
void workspaces_refresh(GtkWidget *workspaces, IO_WorkspacesEvent event);

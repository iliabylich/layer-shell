#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(WorkspaceItem, workspace_item, WORKSPACE, ITEM, GObject)

#define WORKSPACE_ITEM(obj)                                                    \
  G_TYPE_CHECK_INSTANCE_CAST(obj, workspace_item_get_type(), WorkspaceItem)

WorkspaceItem *workspace_item_new(guint num);

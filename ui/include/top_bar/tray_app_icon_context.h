#pragma once

#include <gtk/gtk.h>

typedef struct {
  GtkWidget *tray;
  GActionGroup *action_group;
  GList **pool;
  char *name;
  char *uuid;
} tray_app_icon_context_t;

tray_app_icon_context_t *
tray_app_icon_context_new_root(GtkWidget *tray, GActionGroup *action_group,
                               GList **pool);

tray_app_icon_context_t *
tray_app_icon_context_new_child(tray_app_icon_context_t *parent,
                                size_t child_idx, const char *uuid);

void tray_app_icon_context_free(tray_app_icon_context_t *context);

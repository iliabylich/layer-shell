#include "ui/include/top_bar/workspaces.h"
#include "ui/include/builder.h"
#include "ui/include/top_bar/workspaces_button.h"

#define WORKSPACES_COUNT 10

typedef struct {
  GtkWidget *buttons[WORKSPACES_COUNT];
  workspace_change_f callback;
} data_t;
#define DATA_KEY "data"

GtkWidget *workspaces_init(workspace_change_f callback) {
  GtkWidget *self = top_bar_get_widget("WORKSPACES");

  data_t *data = malloc(sizeof(data_t));
  data->callback = callback;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  for (size_t i = 0; i < WORKSPACES_COUNT; i++) {
    GtkWidget *button = workspaces_button_new(self, i);
    gtk_box_append(GTK_BOX(self), button);
    data->buttons[i] = button;
  }

  return self;
}

void workspaces_emit_switched(GtkWidget *self, size_t idx) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  data->callback(idx);
}

void workspaces_refresh(GtkWidget *self, IO_WorkspacesEvent event) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  for (size_t i = 0; i < WORKSPACES_COUNT; i++) {
    GtkWidget *button = data->buttons[i];
    bool visible = i < 5;
    for (size_t j = 0; j < event.ids.len; j++) {
      if (event.ids.ptr[j] == i + 1) {
        visible = true;
      }
    }
    gtk_widget_set_visible(button, visible);
    if (i + 1 == event.active_id) {
      workspaces_button_make_active(button);
    } else {
      workspaces_button_make_inactive(button);
    }
  }
}

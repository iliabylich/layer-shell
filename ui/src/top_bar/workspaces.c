#include "ui/include/top_bar/workspaces.h"
#include "ui/include/builder.h"
#include "ui/include/top_bar/workspaces_button.h"
#include "ui/include/utils/has_callback.h"

WIDGET_HAS_PROP(buttons, GList *)

#define WORKSPACES_COUNT 10

GtkWidget *workspaces_init(workspace_change_f callback) {
  GtkWidget *self = top_bar_get_widget("WORKSPACES");

  GList *buttons = NULL;
  for (size_t num = 1; num <= WORKSPACES_COUNT; num++) {
    GtkWidget *button = workspaces_button_new(num, callback);
    gtk_box_append(GTK_BOX(self), button);
    buttons = g_list_append(buttons, button);
  }
  set_buttons(self, buttons);

  return self;
}

void workspaces_refresh(GtkWidget *self, IO_WorkspacesEvent event) {
  for (GList *ptr = get_buttons(self); ptr != NULL; ptr = ptr->next) {
    GtkWidget *button = GTK_WIDGET(ptr->data);
    size_t num = workspaces_button_get_number(button);

    bool visible = num <= 5;
    for (size_t j = 0; j < event.workspaces.len; j++) {
      if (event.workspaces.ptr[j] == num) {
        visible = true;
      }
    }
    gtk_widget_set_visible(button, visible);
    if (num == event.active_workspace) {
      workspaces_button_make_active(button);
    } else {
      workspaces_button_make_inactive(button);
    }
  }
}

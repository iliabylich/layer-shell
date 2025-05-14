#include "ui/include/top_bar/workspaces_button.h"

struct _WorkspacesButton {
  GtkButton parent_instance;

  Workspaces *workspaces;
  size_t idx;
};

G_DEFINE_TYPE(WorkspacesButton, workspaces_button, GTK_TYPE_BUTTON)

static void workspaces_button_class_init(WorkspacesButtonClass *) {}

static void workspaces_button_on_click(WorkspacesButton *self) {
  workspaces_emit_switched(self->workspaces, self->idx);
}

static void workspaces_button_init(WorkspacesButton *self) {
  g_signal_connect(self, "clicked", G_CALLBACK(workspaces_button_on_click),
                   NULL);
}

GtkWidget *workspaces_button_new(Workspaces *workspaces, size_t idx) {
  WorkspacesButton *self = g_object_new(WORKSPACES_BUTTON_TYPE, NULL);

  self->workspaces = workspaces;
  self->idx = idx;

  char label[5];
  sprintf(label, "%lu", self->idx + 1);
  gtk_button_set_label(GTK_BUTTON(self), label);

  return GTK_WIDGET(self);
}

void workspaces_button_make_active(WorkspacesButton *self) {
  gtk_widget_add_css_class(GTK_WIDGET(self), "active");
}
void workspaces_button_make_inactive(WorkspacesButton *self) {
  gtk_widget_remove_css_class(GTK_WIDGET(self), "active");
}

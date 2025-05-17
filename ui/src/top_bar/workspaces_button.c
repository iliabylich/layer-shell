#include "ui/include/top_bar/workspaces_button.h"
#include "glib-object.h"
#include "ui/include/top_bar/workspaces.h"

typedef struct {
  GtkWidget *workspaces;
  size_t idx;
} data_t;

#define DATA_KEY "data"

static void on_click(GtkButton *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  workspaces_emit_switched(data->workspaces, data->idx);
}

GtkWidget *workspaces_button_new(GtkWidget *workspaces, size_t idx) {
  GtkWidget *self = gtk_button_new();
  char label[5];
  sprintf(label, "%lu", idx + 1);
  gtk_button_set_label(GTK_BUTTON(self), label);

  data_t *data = malloc(sizeof(data_t));
  data->workspaces = workspaces;
  data->idx = idx;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  g_signal_connect(self, "clicked", G_CALLBACK(on_click), NULL);

  return self;
}

void workspaces_button_make_active(GtkWidget *self) {
  gtk_widget_add_css_class(GTK_WIDGET(self), "active");
}
void workspaces_button_make_inactive(GtkWidget *self) {
  gtk_widget_remove_css_class(GTK_WIDGET(self), "active");
}

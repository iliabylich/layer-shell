#include "ui/include/top_bar/workspaces_button.h"
#include "ui/include/utils/has_callback.h"

WIDGET_HAS_CALLBACK(callback, workspace_change_f)
WIDGET_HAS_PROP(num, size_t)

static void clicked(GtkWidget *self) { get_callback(self)(get_num(self)); }

GtkWidget *workspaces_button_new(size_t num, workspace_change_f callback) {
  GtkWidget *self = gtk_button_new();
  char label[5];
  sprintf(label, "%lu", num);
  gtk_button_set_label(GTK_BUTTON(self), label);

  set_callback(self, callback);
  set_num(self, num);

  g_signal_connect(self, "clicked", G_CALLBACK(clicked), NULL);

  return self;
}

size_t workspaces_button_get_number(GtkWidget *self) { return get_num(self); }

void workspaces_button_make_active(GtkWidget *self) {
  gtk_widget_add_css_class(GTK_WIDGET(self), "active");
}
void workspaces_button_make_inactive(GtkWidget *self) {
  gtk_widget_remove_css_class(GTK_WIDGET(self), "active");
}

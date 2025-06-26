#include "ui/include/top_bar/change_theme.h"
#include "ui/include/builder.h"

GtkWidget *change_theme_init(change_theme_clicked_f callback) {
  GtkWidget *self = top_bar_get_widget("CHANGE_THEME");
  g_signal_connect(self, "clicked", G_CALLBACK(callback), NULL);
  return self;
}

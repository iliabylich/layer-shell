#include "ui/include/top_bar/change_theme.h"
#include "ui/include/builder.h"
#include "ui/include/utils/has_callback.h"

WIDGET_HAS_CALLBACK(on_click_callback, change_theme_clicked_f)

static void on_click(GtkWidget *self) { get_on_click_callback(self)(); }

GtkWidget *change_theme_init(change_theme_clicked_f callback) {
  GtkWidget *self = top_bar_get_widget("CHANGE_THEME");
  set_on_click_callback(self, callback);
  g_signal_connect(self, "clicked", G_CALLBACK(on_click), NULL);
  return self;
}

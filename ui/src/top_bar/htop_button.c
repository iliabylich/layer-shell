#include "ui/include/top_bar/htop_button.h"
#include "ui/include/macros.h"
#include "ui/include/top_bar.h"

GtkWidget *htop_button_init(htop_button_clicked_f callback) {
  GtkWidget *self = top_bar_get_widget_by_id("HTOP_BUTTON");
  g_signal_connect(self, "clicked", callback, NULL);
  return self;
}

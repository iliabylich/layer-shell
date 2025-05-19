#include "ui/include/top_bar/power.h"
#include "ui/include/builder.h"

GtkWidget *power_init(power_clicked_f callback) {
  GtkWidget *self = top_bar_get_widget("POWER");
  g_signal_connect(self, "clicked", callback, NULL);
  return self;
}

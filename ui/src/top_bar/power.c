#include "ui/include/top_bar/power.h"
#include "gtk/gtk.h"
#include "ui/include/icons.h"
#include "ui/include/top_bar.h"

GtkWidget *power_init(power_clicked_f callback) {
  GtkWidget *self = top_bar_get_widget_by_id("POWER");
  GtkWidget *power_icon = top_bar_get_widget_by_id("POWER_ICON");
  gtk_image_set_from_gicon(GTK_IMAGE(power_icon), get_power_icon());
  g_signal_connect(self, "clicked", callback, NULL);
  return self;
}

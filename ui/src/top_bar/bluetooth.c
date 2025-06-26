#include "ui/include/top_bar/bluetooth.h"
#include "ui/include/builder.h"

GtkWidget *bluetooth_init(bluetooth_clicked_f callback) {
  GtkWidget *self = top_bar_get_widget("BLUETOOTH");
  g_signal_connect(self, "clicked", callback, NULL);
  return self;
}

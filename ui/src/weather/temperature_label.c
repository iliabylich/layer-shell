#include "ui/include/weather/temperature_label.h"

GtkWidget *temperature_label_new() { return gtk_label_new("??"); }

void temperature_label_refresh(GtkWidget *label, float temperature) {
  char buffer[100];
  sprintf(buffer, "%5.1f℃", temperature);
  gtk_label_set_label(GTK_LABEL(label), buffer);
}

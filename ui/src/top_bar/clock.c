#include "ui/include/top_bar/clock.h"

GtkWidget *clock_new() {
  GtkWidget *label = gtk_label_new("--");
  gtk_widget_add_css_class(label, "widget");
  gtk_widget_add_css_class(label, "clock");
  gtk_widget_add_css_class(label, "padded");
  gtk_widget_set_name(label, "Clock");
  return label;
}

void clock_refresh(Clock *self, const char *time) {
  gtk_label_set_text(self, time);
}

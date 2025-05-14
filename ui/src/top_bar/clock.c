#include "ui/include/top_bar/clock.h"
#include "gtk/gtk.h"

static const char *css_classes[] = {"widget", "clock", "padded", NULL};

GtkWidget *clock_new() {
  GtkWidget *label = gtk_label_new("--");
  gtk_widget_set_css_classes(label, css_classes);
  gtk_widget_set_name(label, "Clock");
  return label;
}

void clock_refresh(Clock *self, const char *time) {
  gtk_label_set_text(self, time);
}

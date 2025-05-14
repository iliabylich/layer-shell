#include "ui/include/top_bar/clock.h"
#include "gtk/gtk.h"

GtkWidget *clock_new() {
  return g_object_new(GTK_TYPE_LABEL,
                      //
                      "label", "--",
                      //
                      "css-classes",
                      (const char *[]){"widget", "clock", "padded", NULL},
                      //
                      "name", "Clock",
                      //
                      NULL);
}

void clock_refresh(Clock *self, const char *time) {
  gtk_label_set_text(self, time);
}

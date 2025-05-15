#include "ui/include/top_bar/clock.h"

GtkWidget *clock_new() {
  // clang-format off
  return g_object_new(
      GTK_TYPE_LABEL,
      "label", "--",
      "css-classes", (const char *[]){"widget", "clock", "padded", NULL},
      "name", "Clock",
      NULL);
  // clang-format on
}

void clock_refresh(Clock *self, const char *time) {
  gtk_label_set_text(self, time);
}

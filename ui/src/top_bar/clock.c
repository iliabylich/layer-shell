#include "ui/include/top_bar/clock.h"
#include "ui/include/macros.h"

GtkWidget *clock_new() {
  // clang-format off
  return g_object_new(
      GTK_TYPE_LABEL,
      "label", "--",
      "css-classes", CSS("widget", "clock", "padded"),
      "name", "Clock",
      NULL);
  // clang-format on
}

void clock_refresh(Clock *self, const char *time) {
  gtk_label_set_text(self, time);
}

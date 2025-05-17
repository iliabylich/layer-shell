#include "ui/include/top_bar/clock.h"
#include "ui/include/top_bar.h"

GtkWidget *clock_init() { return top_bar_get_widget_by_id("CLOCK"); }

void clock_refresh(GtkWidget *self, const char *time) {
  gtk_label_set_text(GTK_LABEL(self), time);
}

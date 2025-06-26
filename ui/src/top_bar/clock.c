#include "ui/include/top_bar/clock.h"
#include "ui/include/builder.h"

GtkWidget *clock_init() { return top_bar_get_widget("CLOCK"); }

void clock_refresh(GtkWidget *self, IO_ClockEvent event) {
  gtk_label_set_text(GTK_LABEL(self), event.time);
}

#include "ui/include/top_bar/memory.h"
#include "ui/include/top_bar.h"

GtkWidget *memory_init(memory_clicked_f callback) {
  GtkWidget *self = top_bar_get_widget_by_id("MEMORY");
  g_signal_connect(self, "clicked", callback, NULL);
  return self;
}

void memory_refresh(GtkWidget *self, double used, double total) {
  char buffer[100];
  sprintf(buffer, "RAM %.1fG/%.1fG", used, total);
  gtk_button_set_label(GTK_BUTTON(self), buffer);
}

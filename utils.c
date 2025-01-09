#include "utils.h"

void flip_window_visibility(GtkWindow *window) {
  gtk_widget_set_visible(GTK_WIDGET(window),
                         !gtk_widget_get_visible(GTK_WIDGET(window)));
}

#include "ui/include/top_bar/change_theme.h"
#include "ui/include/icons.h"
#include "ui/include/macros.h"
#include "ui/include/top_bar.h"

typedef struct {
  change_theme_clicked_f callback;
} data_t;
#define DATA_KEY "data"

static void on_click(GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  data->callback();
}

GtkWidget *change_theme_init(change_theme_clicked_f callback) {
  GtkWidget *self = top_bar_get_widget_by_id("CHANGE_THEME");

  GtkWidget *image = top_bar_get_widget_by_id("CHANGE_THEME_IMAGE");
  gtk_image_set_from_gicon(GTK_IMAGE(image), get_change_theme_icon());

  data_t *data = malloc(sizeof(data_t));
  data->callback = callback;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  g_signal_connect(self, "clicked", G_CALLBACK(on_click), data);

  return self;
}

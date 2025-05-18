#include "ui/include/top_bar/sound.h"
#include "ui/include/builder.h"

typedef struct {
  GtkWidget *image;
} data_t;
#define DATA_KEY "data"

GtkWidget *sound_init() {
  GtkWidget *self = top_bar_get_widget("SOUND");
  GtkWidget *image = top_bar_get_widget("SOUND_IMAGE");

  data_t *data = malloc(sizeof(data_t));
  data->image = image;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  return self;
}

void sound_refresh(GtkWidget *self, uint32_t volume, bool muted) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  const char *icon_name = NULL;
  if (volume == 0 || muted) {
    icon_name = "audio-volume-muted-symbolic";
  } else if (volume >= 1 && volume < 34) {
    icon_name = "audio-volume-low-symbolic";
  } else if (volume >= 34 && volume < 67) {
    icon_name = "audio-volume-medium-symbolic";
  } else if (volume >= 67 && volume < 95) {
    icon_name = "audio-volume-high-symbolic";
  } else {
    icon_name = "audio-volume-overamplified-symbolic";
  }
  gtk_image_set_from_icon_name(GTK_IMAGE(data->image), icon_name);
}

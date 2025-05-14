#include "ui/include/top_bar/sound.h"
#include "gtk/gtk.h"

struct _Sound {
  GtkBox parent_instance;

  GtkWidget *image;
};

G_DEFINE_TYPE(Sound, sound, GTK_TYPE_BOX)

static void sound_class_init(SoundClass *) {}

static void sound_init(Sound *self) {
  self->image = gtk_image_new_from_icon_name("dialog-question");
  gtk_box_append(GTK_BOX(self), self->image);
}

GtkWidget *sound_new() {
  return g_object_new(sound_get_type(),
                      //
                      "orientation", GTK_ORIENTATION_HORIZONTAL,
                      //
                      "spacing", 5,
                      //
                      "css-classes",
                      (const char *[]){"widget", "sound", "padded", NULL},
                      //
                      "name", "Sound",
                      //
                      NULL);
}

void sound_refresh(Sound *self, uint32_t volume, bool muted) {
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
  gtk_image_set_from_icon_name(GTK_IMAGE(self->image), icon_name);
}

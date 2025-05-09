#include "ui/include/top_bar/sound.h"

struct _Sound {
  GtkBox parent_instance;

  GtkWidget *image;
};

G_DEFINE_TYPE(Sound, sound, GTK_TYPE_BOX)

static void sound_class_init(SoundClass *) {}

static void sound_init(Sound *self) {
  gtk_orientable_set_orientation(GTK_ORIENTABLE(self),
                                 GTK_ORIENTATION_HORIZONTAL);
  gtk_box_set_spacing(GTK_BOX(self), 5);
  gtk_widget_add_css_class(GTK_WIDGET(self), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(self), "sound");
  gtk_widget_add_css_class(GTK_WIDGET(self), "padded");
  gtk_widget_set_name(GTK_WIDGET(self), "Sound");

  self->image = gtk_image_new_from_icon_name("dialog-question");
  gtk_box_append(GTK_BOX(self), self->image);
}

GtkWidget *sound_new() { return g_object_new(sound_get_type(), NULL); }

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

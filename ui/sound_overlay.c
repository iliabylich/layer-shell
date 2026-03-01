#include "ui/sound_overlay.h"
#include "ui/logger.h"

LOGGER("SoundOverlay", 0)

struct _SoundOverlay {
  GtkWidget parent_instance;
};

G_DEFINE_TYPE(SoundOverlay, sound_overlay, BASE_OVERLAY_TYPE)

static const char *volume_to_icon(uint32_t volume, bool muted) {
  if (volume == 0 || muted) {
    return "󰝟";
  } else if (volume <= 33) {
    return "󰕿";
  } else if (volume <= 66) {
    return "󰖀";
  } else {
    return "󰕾";
  }
}

static char *format_sound_icon(GObject *, guint volume, bool muted) {
  return g_strdup(volume_to_icon(volume, muted));
}

static void sound_overlay_init(SoundOverlay *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));
}

static void sound_overlay_class_init(SoundOverlayClass *klass) {
  LOG("class init");

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(widget_class,
                                              "/layer-shell/sound_overlay.ui");
  gtk_widget_class_bind_template_callback(widget_class, format_sound_icon);
}

GtkWidget *sound_overlay_new(GtkApplication *app, IOModel *model) {
  return g_object_new(sound_overlay_get_type(), "application", app, "model",
                      model, NULL);
}

#include "ui/sound_overlay.h"
#include "ui/logger.h"

LOGGER("SoundOverlay", 0)

struct _SoundOverlay {
  GtkWidget parent_instance;

  IOModel *model;
  GtkAdjustment *sound_adjustment;
};

G_DEFINE_TYPE(SoundOverlay, sound_overlay, BASE_OVERLAY_TYPE)

enum {
  PROP_MODEL = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

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

static double normalized_volume(guint volume) {
  if (volume == 99)
    return 100.0;
  return volume;
}

static gboolean transform_sound_volume(GBinding *, const GValue *from_value,
                                       GValue *to_value, gpointer) {
  g_value_set_double(to_value, normalized_volume(g_value_get_uint(from_value)));
  return true;
}

static void sound_overlay_get_property(GObject *object, guint property_id,
                                       GValue *value, GParamSpec *pspec) {
  SoundOverlay *self = SOUND_OVERLAY(object);
  switch (property_id) {
  case PROP_MODEL:
    g_value_set_object(value, self->model);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void sound_overlay_set_property(GObject *object, guint property_id,
                                       const GValue *value, GParamSpec *pspec) {
  SoundOverlay *self = SOUND_OVERLAY(object);
  switch (property_id) {
  case PROP_MODEL: {
    GObject *sound = NULL;
    g_set_object(&self->model, g_value_get_object(value));
    g_object_get(self->model, "sound", &sound, NULL);
    g_object_bind_property_full(sound, "volume", self->sound_adjustment,
                                "value", G_BINDING_SYNC_CREATE,
                                transform_sound_volume, NULL, NULL, NULL);
    g_object_unref(sound);
    break;
  }
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void sound_overlay_init(SoundOverlay *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));
  self->model = NULL;
}

static void sound_overlay_dispose(GObject *object) {
  LOG("dispose");
  SoundOverlay *self = SOUND_OVERLAY(object);
  g_clear_object(&self->model);
  G_OBJECT_CLASS(sound_overlay_parent_class)->dispose(object);
}

static void sound_overlay_class_init(SoundOverlayClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = sound_overlay_get_property;
  object_class->set_property = sound_overlay_set_property;
  object_class->dispose = sound_overlay_dispose;

  properties[PROP_MODEL] =
      g_param_spec_object("model", NULL, NULL, io_model_get_type(),
                          G_PARAM_READWRITE | G_PARAM_CONSTRUCT_ONLY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(widget_class,
                                              "/layer-shell/sound_overlay.ui");
  gtk_widget_class_bind_template_child(widget_class, SoundOverlay,
                                       sound_adjustment);
  gtk_widget_class_bind_template_callback(widget_class, format_sound_icon);
}

GtkWidget *sound_overlay_new(GtkApplication *app, IOModel *model) {
  return g_object_new(sound_overlay_get_type(), "application", app, "model",
                      model, NULL);
}

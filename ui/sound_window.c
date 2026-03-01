#include "ui/sound_window.h"
#include "ui/logger.h"

LOGGER("SoundWindow", 0)

struct _SoundWindow {
  GtkWidget parent_instance;

  IOModel *model;
  SoundWindowModel *window_model;
  GtkAdjustment *sound_adjustment;
  bool sound_initialized;

  guint timer;
};

G_DEFINE_TYPE(SoundWindow, sound_window, BASE_WINDOW_TYPE)

enum {
  PROP_MODEL = 1,
  PROP_WINDOW_MODEL,
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

static void hide(gpointer data) {
  SoundWindow *self = data;
  g_object_set(self->window_model, "visible", false, NULL);
  self->timer = 0;
}

static void show(SoundWindow *self) {
  if (!self->sound_initialized)
    return;

  g_object_set(self->window_model, "visible", true, NULL);

  if (self->timer) {
    g_assert(g_source_remove(self->timer));
  }
  self->timer = g_timeout_add_once(1000, hide, self);
}

static void sound_value_changed(GObject *, GParamSpec *, gpointer data) {
  SoundWindow *self = SOUND_WINDOW(data);
  show(self);
}

static void sound_ready_changed(GObject *, GParamSpec *, gpointer data) {
  SoundWindow *self = SOUND_WINDOW(data);
  gboolean ready = false;
  g_object_get(self->model, "sound-ready", &ready, NULL);
  self->sound_initialized = ready;
}

static void sound_window_get_property(GObject *object, guint property_id,
                                      GValue *value, GParamSpec *pspec) {
  SoundWindow *self = SOUND_WINDOW(object);
  switch (property_id) {
  case PROP_MODEL:
    g_value_set_object(value, self->model);
    break;
  case PROP_WINDOW_MODEL:
    g_value_set_object(value, self->window_model);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void sound_window_set_property(GObject *object, guint property_id,
                                      const GValue *value, GParamSpec *pspec) {
  SoundWindow *self = SOUND_WINDOW(object);
  switch (property_id) {
  case PROP_MODEL:
    g_set_object(&self->model, g_value_get_object(value));
    g_object_bind_property_full(
        self->model, "sound-volume", self->sound_adjustment, "value",
        G_BINDING_SYNC_CREATE, transform_sound_volume, NULL, NULL, NULL);
    sound_ready_changed(NULL, NULL, self);
    g_signal_connect_object(self->model, "notify::sound-ready",
                            G_CALLBACK(sound_ready_changed), self, 0);
    g_signal_connect_object(self->model, "notify::sound-volume",
                            G_CALLBACK(sound_value_changed), self, 0);
    g_signal_connect_object(self->model, "notify::sound-muted",
                            G_CALLBACK(sound_value_changed), self, 0);
    break;
  case PROP_WINDOW_MODEL:
    g_set_object(&self->window_model, g_value_get_object(value));
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void sound_window_init(SoundWindow *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));
  self->model = NULL;
  self->window_model = NULL;
  self->sound_initialized = false;
}

static void sound_window_dispose(GObject *object) {
  LOG("dispose");
  SoundWindow *self = SOUND_WINDOW(object);
  g_clear_object(&self->model);
  g_clear_object(&self->window_model);
  G_OBJECT_CLASS(sound_window_parent_class)->dispose(object);
}

static void sound_window_class_init(SoundWindowClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = sound_window_get_property;
  object_class->set_property = sound_window_set_property;
  object_class->dispose = sound_window_dispose;

  properties[PROP_MODEL] =
      g_param_spec_object("model", NULL, NULL, io_model_get_type(),
                          G_PARAM_READWRITE | G_PARAM_CONSTRUCT_ONLY);
  properties[PROP_WINDOW_MODEL] = g_param_spec_object(
      "window-model", NULL, NULL, sound_window_model_get_type(),
      G_PARAM_READWRITE | G_PARAM_CONSTRUCT_ONLY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(widget_class,
                                              "/layer-shell/sound_window.ui");
  gtk_widget_class_bind_template_child(widget_class, SoundWindow,
                                       sound_adjustment);
  gtk_widget_class_bind_template_callback(widget_class, format_sound_icon);
}

GtkWidget *sound_window_new(GtkApplication *app, IOModel *model,
                            SoundWindowModel *window_model) {
  return g_object_new(sound_window_get_type(), "application", app, "model",
                      model, "window-model", window_model, NULL);
}

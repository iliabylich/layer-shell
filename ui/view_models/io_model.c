#include "ui/view_models/io_model.h"
#include "ui/view_models/caps_lock_model.h"
#include "ui/view_models/clock_model.h"
#include "ui/view_models/cpu_model.h"
#include "ui/view_models/language_model.h"
#include "ui/view_models/memory_model.h"
#include "ui/view_models/network_model.h"
#include "ui/view_models/overlays_model.h"
#include "ui/view_models/sound_model.h"
#include "ui/view_models/tray_model.h"
#include "ui/view_models/weather_model.h"
#include "ui/view_models/workspaces_model.h"

struct _IOModel {
  GObject parent_instance;

  ClockModel *clock;
  LanguageModel *language;
  MemoryModel *memory;
  WeatherModel *weather;
  NetworkModel *network;
  OverlaysModel *overlays;
  SoundModel *sound;
  CapsLockModel *caps_lock;
  CpuModel *cpu;
  WorkspacesModel *workspaces;
  TrayModel *tray;
};

G_DEFINE_TYPE(IOModel, io_model, G_TYPE_OBJECT)

enum {
  PROP_WORKSPACES = 1,
  PROP_TRAY,
  PROP_OVERLAYS,
  PROP_CPU,
  PROP_CLOCK,
  PROP_LANGUAGE,
  PROP_MEMORY,
  PROP_NETWORK,
  PROP_SOUND,
  PROP_CAPS_LOCK,
  PROP_WEATHER,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void sound_overlay_visibility_changed(SoundModel *, gboolean visible,
                                             gpointer data) {
  IOModel *self = IO_MODEL(data);
  g_object_set(self->overlays, "sound", visible, NULL);
}

static void caps_lock_overlay_visibility_changed(CapsLockModel *,
                                                 gboolean visible,
                                                 gpointer data) {
  IOModel *self = IO_MODEL(data);
  g_object_set(self->overlays, "caps-lock", visible, NULL);
}

static void io_model_get_property(GObject *object, guint property_id,
                                  GValue *value, GParamSpec *pspec) {
  IOModel *self = IO_MODEL(object);

  switch (property_id) {
  case PROP_OVERLAYS:
    g_value_set_object(value, self->overlays);
    break;
  case PROP_CPU:
    g_value_set_object(value, self->cpu);
    break;
  case PROP_CLOCK:
    g_value_set_object(value, self->clock);
    break;
  case PROP_LANGUAGE:
    g_value_set_object(value, self->language);
    break;
  case PROP_MEMORY:
    g_value_set_object(value, self->memory);
    break;
  case PROP_NETWORK:
    g_value_set_object(value, self->network);
    break;
  case PROP_SOUND:
    g_value_set_object(value, self->sound);
    break;
  case PROP_CAPS_LOCK:
    g_value_set_object(value, self->caps_lock);
    break;
  case PROP_WEATHER:
    g_value_set_object(value, self->weather);
    break;
  case PROP_WORKSPACES:
    g_value_set_object(value, self->workspaces);
    break;
  case PROP_TRAY:
    g_value_set_object(value, self->tray);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void io_model_set_property(GObject *object, guint property_id,
                                  const GValue *value, GParamSpec *pspec) {
  (void)value;
  G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
}

static void io_model_finalize(GObject *object) {
  IOModel *self = IO_MODEL(object);
  g_clear_object(&self->clock);
  g_clear_object(&self->language);
  g_clear_object(&self->memory);
  g_clear_object(&self->network);
  g_clear_object(&self->overlays);
  g_clear_object(&self->weather);
  g_clear_object(&self->sound);
  g_clear_object(&self->caps_lock);
  g_clear_object(&self->cpu);
  g_clear_object(&self->workspaces);
  g_clear_object(&self->tray);
  G_OBJECT_CLASS(io_model_parent_class)->finalize(object);
}

static void io_model_init(IOModel *self) {
  self->clock = clock_model_new();
  self->language = language_model_new();
  self->memory = memory_model_new();
  self->weather = weather_model_new();
  self->network = network_model_new();
  self->overlays = overlays_model_new();
  self->sound = sound_model_new();
  self->caps_lock = caps_lock_model_new();

  self->cpu = cpu_model_new();
  self->workspaces = workspaces_model_new();
  self->tray = tray_model_new();

  g_signal_connect_object(self->sound, "overlay-visibility-changed",
                          G_CALLBACK(sound_overlay_visibility_changed), self, 0);
  g_signal_connect_object(self->caps_lock, "overlay-visibility-changed",
                          G_CALLBACK(caps_lock_overlay_visibility_changed), self,
                          0);
}

static void io_model_class_init(IOModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = io_model_get_property;
  object_class->set_property = io_model_set_property;
  object_class->finalize = io_model_finalize;

  properties[PROP_WORKSPACES] =
      g_param_spec_object("workspaces", NULL, NULL, workspaces_model_get_type(),
                          G_PARAM_READABLE);
  properties[PROP_TRAY] =
      g_param_spec_object("tray", NULL, NULL, tray_model_get_type(),
                          G_PARAM_READABLE);
  properties[PROP_OVERLAYS] =
      g_param_spec_object("overlays", NULL, NULL, overlays_model_get_type(),
                          G_PARAM_READABLE);
  properties[PROP_CPU] = g_param_spec_object("cpu", NULL, NULL,
                                             cpu_model_get_type(),
                                             G_PARAM_READABLE);
  properties[PROP_CLOCK] =
      g_param_spec_object("clock", NULL, NULL, clock_model_get_type(),
                          G_PARAM_READABLE);
  properties[PROP_LANGUAGE] =
      g_param_spec_object("language", NULL, NULL, language_model_get_type(),
                          G_PARAM_READABLE);
  properties[PROP_MEMORY] =
      g_param_spec_object("memory", NULL, NULL, memory_model_get_type(),
                          G_PARAM_READABLE);
  properties[PROP_NETWORK] =
      g_param_spec_object("network", NULL, NULL, network_model_get_type(),
                          G_PARAM_READABLE);
  properties[PROP_SOUND] =
      g_param_spec_object("sound", NULL, NULL, sound_model_get_type(),
                          G_PARAM_READABLE);
  properties[PROP_CAPS_LOCK] =
      g_param_spec_object("caps-lock", NULL, NULL, caps_lock_model_get_type(),
                          G_PARAM_READABLE);
  properties[PROP_WEATHER] =
      g_param_spec_object("weather", NULL, NULL, weather_model_get_type(),
                          G_PARAM_READABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

IOModel *io_model_new(void) { return g_object_new(io_model_get_type(), NULL); }

void io_model_tray_add_app(IOModel *self, const char *service, IO_TrayIcon icon,
                           IO_FFIArray_TrayItem items) {
  tray_model_add_app(self->tray, service, icon, items);
}

void io_model_tray_remove_app(IOModel *self, const char *service) {
  tray_model_remove_app(self->tray, service);
}

void io_model_tray_set_icon(IOModel *self, const char *service,
                            IO_TrayIcon icon) {
  tray_model_update_icon(self->tray, service, icon);
}

void io_model_tray_set_menu(IOModel *self, const char *service,
                            IO_FFIArray_TrayItem items) {
  tray_model_update_menu(self->tray, service, items);
}

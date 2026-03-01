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
  guint sound_overlay_timer;
  CapsLockModel *caps_lock;
  CpuModel *cpu;
  WorkspacesModel *workspaces;
  TrayModel *tray;
};

G_DEFINE_TYPE(IOModel, io_model, G_TYPE_OBJECT)

enum {
  PROP_CLOCK_UNIX_SECONDS = 1,
  PROP_LANGUAGE_TEXT,
  PROP_MEMORY_USED,
  PROP_MEMORY_TOTAL,
  PROP_WEATHER_TEXT,
  PROP_WEATHER_OVERLAY_VISIBLE,
  PROP_NETWORK_SSID,
  PROP_NETWORK_STRENGTH,
  PROP_NETWORK_NAME,
  PROP_DOWNLOAD_BYTES_PER_SEC,
  PROP_UPLOAD_BYTES_PER_SEC,
  PROP_SOUND_VOLUME,
  PROP_SOUND_MUTED,
  PROP_SOUND_OVERLAY_VISIBLE,
  PROP_SESSION_OVERLAY_VISIBLE,
  PROP_TERMINAL_OVERLAY_VISIBLE,
  PROP_PING_OVERLAY_VISIBLE,
  PROP_CAPS_LOCK_ENABLED,
  PROP_CAPS_LOCK_VISIBLE,
  PROP_WEATHER_HOURLY_FORECAST,
  PROP_WEATHER_DAILY_FORECAST,
  PROP_WORKSPACES,
  PROP_CPU_CORES,
  PROP_TRAY_APPS,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

// Proxy definitions for io_model_{get,set}_property forwarding.
#define FOREACH_PROXY_READWRITE(X)                                              \
  X(PROP_CLOCK_UNIX_SECONDS, clock, "unix_seconds")                            \
  X(PROP_LANGUAGE_TEXT, language, "text")                                      \
  X(PROP_MEMORY_USED, memory, "used")                                          \
  X(PROP_MEMORY_TOTAL, memory, "total")                                        \
  X(PROP_WEATHER_OVERLAY_VISIBLE, overlays, "weather_visible")                 \
  X(PROP_NETWORK_SSID, network, "ssid")                                        \
  X(PROP_NETWORK_STRENGTH, network, "strength")                                \
  X(PROP_DOWNLOAD_BYTES_PER_SEC, network, "download_bytes_per_sec")            \
  X(PROP_UPLOAD_BYTES_PER_SEC, network, "upload_bytes_per_sec")                \
  X(PROP_SOUND_VOLUME, sound, "volume")                                        \
  X(PROP_SOUND_MUTED, sound, "muted")                                          \
  X(PROP_SOUND_OVERLAY_VISIBLE, overlays, "sound_visible")                     \
  X(PROP_SESSION_OVERLAY_VISIBLE, overlays, "session_visible")                 \
  X(PROP_TERMINAL_OVERLAY_VISIBLE, overlays, "terminal_visible")               \
  X(PROP_PING_OVERLAY_VISIBLE, overlays, "ping_visible")                       \
  X(PROP_CAPS_LOCK_ENABLED, caps_lock, "enabled")                              \
  X(PROP_CAPS_LOCK_VISIBLE, overlays, "caps_lock_visible")

#define FOREACH_PROXY_READONLY(X)                                               \
  X(PROP_WEATHER_TEXT, weather, "text")                                        \
  X(PROP_NETWORK_NAME, network, "name")                                        \
  X(PROP_WEATHER_HOURLY_FORECAST, weather, "hourly_forecast")                  \
  X(PROP_WEATHER_DAILY_FORECAST, weather, "daily_forecast")                    \
  X(PROP_WORKSPACES, workspaces, "visible")                                    \
  X(PROP_CPU_CORES, cpu, "cores")                                              \
  X(PROP_TRAY_APPS, tray, "apps")

#define FOREACH_GETTER(X)                                                       \
  FOREACH_PROXY_READWRITE(X)                                                    \
  FOREACH_PROXY_READONLY(X)

#define FOREACH_SETTER(X) FOREACH_PROXY_READWRITE(X)

#define DEFINE_NOTIFY(func_name, prop_id)                                \
  static void func_name(GObject *, GParamSpec *, gpointer data) {              \
    IOModel *self = IO_MODEL(data);                                             \
    g_object_notify_by_pspec(G_OBJECT(self), properties[prop_id]);              \
  }

#define FOREACH_PROXIED_NOTIFY(X)                                              \
  X(clock, unix_seconds, PROP_CLOCK_UNIX_SECONDS)                              \
  X(language, text, PROP_LANGUAGE_TEXT)                                        \
  X(weather, text, PROP_WEATHER_TEXT)                                          \
  X(memory, used, PROP_MEMORY_USED)                                            \
  X(memory, total, PROP_MEMORY_TOTAL)                                          \
  X(network, ssid, PROP_NETWORK_SSID)                                          \
  X(network, strength, PROP_NETWORK_STRENGTH)                                  \
  X(network, name, PROP_NETWORK_NAME)                                          \
  X(network, download_bytes_per_sec, PROP_DOWNLOAD_BYTES_PER_SEC)              \
  X(network, upload_bytes_per_sec, PROP_UPLOAD_BYTES_PER_SEC)                  \
  X(overlays, weather_visible, PROP_WEATHER_OVERLAY_VISIBLE)                   \
  X(overlays, session_visible, PROP_SESSION_OVERLAY_VISIBLE)                   \
  X(overlays, terminal_visible, PROP_TERMINAL_OVERLAY_VISIBLE)                 \
  X(overlays, ping_visible, PROP_PING_OVERLAY_VISIBLE)                         \
  X(overlays, sound_visible, PROP_SOUND_OVERLAY_VISIBLE)                       \
  X(sound, volume, PROP_SOUND_VOLUME)                                          \
  X(sound, muted, PROP_SOUND_MUTED)                                            \
  X(caps_lock, enabled, PROP_CAPS_LOCK_ENABLED)                                \
  X(overlays, caps_lock_visible, PROP_CAPS_LOCK_VISIBLE)

#define DEFINE_PROXIED_NOTIFY(field, prop, prop_id)                            \
  DEFINE_NOTIFY(notify_##field##_##prop, prop_id)
FOREACH_PROXIED_NOTIFY(DEFINE_PROXIED_NOTIFY)
#undef DEFINE_PROXIED_NOTIFY

static void connect_proxy_notify(GObject *object, const char *prop_name,
                                 GCallback callback, gpointer data) {
  char *signal_name = g_strconcat("notify::", prop_name, NULL);
  g_strdelimit(signal_name, "_", '-');
  g_signal_connect_object(object, signal_name, callback, data, 0);
  g_free(signal_name);
}

static void hide_sound_overlay(gpointer data) {
  IOModel *self = IO_MODEL(data);
  self->sound_overlay_timer = 0;
  g_object_set(self->overlays, "sound_visible", false, NULL);
}

static void sound_overlay_show_requested(SoundModel *, gpointer data) {
  IOModel *self = IO_MODEL(data);
  g_object_set(self->overlays, "sound_visible", true, NULL);
  if (self->sound_overlay_timer != 0) {
    g_assert(g_source_remove(self->sound_overlay_timer));
  }
  self->sound_overlay_timer = g_timeout_add_once(1000, hide_sound_overlay, self);
}

static void io_model_get_property(GObject *object, guint property_id,
                                  GValue *value, GParamSpec *pspec) {
  IOModel *self = IO_MODEL(object);

#define IO_MODEL_GET_PROXY_CASE(prop_id, field, prop_name)                     \
  case prop_id:                                                                 \
    g_object_get_property(G_OBJECT(self->field), prop_name, value);            \
    break;

  switch (property_id) {
    FOREACH_GETTER(IO_MODEL_GET_PROXY_CASE)
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }

#undef IO_MODEL_GET_PROXY_CASE
}

static void io_model_set_property(GObject *object, guint property_id,
                                  const GValue *value, GParamSpec *pspec) {
  IOModel *self = IO_MODEL(object);

#define IO_MODEL_SET_PROXY_CASE(prop_id, field, prop_name)                     \
  case prop_id:                                                                 \
    g_object_set_property(G_OBJECT(self->field), prop_name, value);            \
    break;

  switch (property_id) {
    FOREACH_SETTER(IO_MODEL_SET_PROXY_CASE)
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }

#undef IO_MODEL_SET_PROXY_CASE
}

static void io_model_finalize(GObject *object) {
  IOModel *self = IO_MODEL(object);
  if (self->sound_overlay_timer != 0) {
    g_assert(g_source_remove(self->sound_overlay_timer));
    self->sound_overlay_timer = 0;
  }
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
  self->sound_overlay_timer = 0;
  self->caps_lock = caps_lock_model_new();

  self->cpu = cpu_model_new();
  self->workspaces = workspaces_model_new();
  self->tray = tray_model_new();

#define CONNECT_PROXIED_NOTIFY(field, prop, prop_id)                           \
  connect_proxy_notify(G_OBJECT(self->field), #prop,                           \
                       G_CALLBACK(notify_##field##_##prop), self);
  FOREACH_PROXIED_NOTIFY(CONNECT_PROXIED_NOTIFY)
#undef CONNECT_PROXIED_NOTIFY

  g_signal_connect_object(self->sound, "overlay-show-requested",
                          G_CALLBACK(sound_overlay_show_requested), self, 0);
}

static void io_model_class_init(IOModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = io_model_get_property;
  object_class->set_property = io_model_set_property;
  object_class->finalize = io_model_finalize;

  properties[PROP_CLOCK_UNIX_SECONDS] =
      g_param_spec_int64("clock_unix_seconds", NULL, NULL, 0, G_MAXINT64, 0,
                         G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_LANGUAGE_TEXT] =
      g_param_spec_string("language_text", NULL, NULL, "--",
                          G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_MEMORY_USED] =
      g_param_spec_double("memory_used", NULL, NULL, 0.0, G_MAXDOUBLE, 0.0,
                          G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_MEMORY_TOTAL] =
      g_param_spec_double("memory_total", NULL, NULL, 0.0, G_MAXDOUBLE, 0.0,
                          G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_WEATHER_TEXT] =
      g_param_spec_string("weather_text", NULL, NULL, "--", G_PARAM_READABLE);
  properties[PROP_WEATHER_OVERLAY_VISIBLE] =
      g_param_spec_boolean("overlays_weather_visible", NULL, NULL, false,
                           G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_NETWORK_SSID] =
      g_param_spec_string("network_ssid", NULL, NULL, NULL,
                          G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_NETWORK_STRENGTH] =
      g_param_spec_uchar("network_strength", NULL, NULL, 0, G_MAXUINT8, 0,
                         G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_NETWORK_NAME] =
      g_param_spec_string("network_name", NULL, NULL, "--", G_PARAM_READABLE);
  properties[PROP_DOWNLOAD_BYTES_PER_SEC] = g_param_spec_uint64(
      "network_download_bytes_per_sec", NULL, NULL, 0, G_MAXUINT64, G_MAXUINT64,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_UPLOAD_BYTES_PER_SEC] = g_param_spec_uint64(
      "network_upload_bytes_per_sec", NULL, NULL, 0, G_MAXUINT64, G_MAXUINT64,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_SOUND_VOLUME] =
      g_param_spec_uint("sound_volume", NULL, NULL, 0, G_MAXUINT, 0,
                        G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_SOUND_MUTED] =
      g_param_spec_boolean("sound_muted", NULL, NULL, false,
                           G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_SOUND_OVERLAY_VISIBLE] =
      g_param_spec_boolean("overlays_sound_visible", NULL, NULL, false,
                           G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_SESSION_OVERLAY_VISIBLE] =
      g_param_spec_boolean("overlays_session_visible", NULL, NULL, false,
                           G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_TERMINAL_OVERLAY_VISIBLE] =
      g_param_spec_boolean("overlays_terminal_visible", NULL, NULL, false,
                           G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_PING_OVERLAY_VISIBLE] =
      g_param_spec_boolean("overlays_ping_visible", NULL, NULL, false,
                           G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_CAPS_LOCK_ENABLED] =
      g_param_spec_boolean("caps_lock_enabled", NULL, NULL, false,
                           G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_CAPS_LOCK_VISIBLE] =
      g_param_spec_boolean("overlays_caps_lock_visible", NULL, NULL, false,
                           G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_WEATHER_HOURLY_FORECAST] =
      g_param_spec_object("weather_hourly_forecast", NULL, NULL,
                          G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_WEATHER_DAILY_FORECAST] =
      g_param_spec_object("weather_daily_forecast", NULL, NULL,
                          G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_WORKSPACES] = g_param_spec_object(
      "workspaces_visible", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_CPU_CORES] = g_param_spec_object(
      "cpu_cores", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_TRAY_APPS] = g_param_spec_object(
      "tray_apps", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

IOModel *io_model_new(void) { return g_object_new(io_model_get_type(), NULL); }

void io_model_set_workspaces(IOModel *self,
                             struct IO_FFIArray_HyprlandWorkspace data) {
  workspaces_model_update(self->workspaces, data);
}

void io_model_set_weather(IOModel *self,
                          struct IO_Event_IO_Weather_Body weather) {
  weather_model_set_weather(self->weather, weather);
}

void io_model_set_cpu(IOModel *self, IO_FFIArray_u8 data) {
  cpu_model_update(self->cpu, data);
}

void io_model_set_initial_sound(IOModel *self, guint volume, gboolean muted) {
  sound_model_set_initial(self->sound, volume, muted);
}

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

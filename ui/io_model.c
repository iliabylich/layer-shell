#include "ui/io_model.h"
#include "ui/cpu_model.h"
#include "ui/tray_model.h"
#include "ui/weather_day_item.h"
#include "ui/weather_helper.h"
#include "ui/weather_hour_item.h"
#include "ui/workspaces_model.h"
#include <string.h>

struct _IOModel {
  GObject parent_instance;

  int64_t clock_unix_seconds;
  char *language_text;
  double memory_used;
  double memory_total;
  char *weather_text;
  char *network_name;
  uint64_t download_bytes_per_sec;
  uint64_t upload_bytes_per_sec;
  uint32_t sound_volume;
  gboolean sound_muted;
  gboolean sound_ready;
  gboolean caps_lock_enabled;
  CpuModel *cpu;
  GListStore *weather_hourly_forecast;
  GListStore *weather_daily_forecast;
  WorkspacesModel *workspaces;
  TrayModel *tray;

  char *network_ssid;
  uint8_t network_strength;
};

G_DEFINE_TYPE(IOModel, io_model, G_TYPE_OBJECT)

enum {
  PROP_CLOCK_UNIX_SECONDS = 1,
  PROP_LANGUAGE_TEXT,
  PROP_MEMORY_USED,
  PROP_MEMORY_TOTAL,
  PROP_WEATHER_TEXT,
  PROP_NETWORK_NAME,
  PROP_DOWNLOAD_BYTES_PER_SEC,
  PROP_UPLOAD_BYTES_PER_SEC,
  PROP_SOUND_VOLUME,
  PROP_SOUND_MUTED,
  PROP_SOUND_READY,
  PROP_CAPS_LOCK_ENABLED,
  PROP_WEATHER_HOURLY_FORECAST,
  PROP_WEATHER_DAILY_FORECAST,
  PROP_WORKSPACES,
  PROP_CPU_CORES,
  PROP_TRAY_APPS,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void io_model_get_property(GObject *object, guint property_id,
                                  GValue *value, GParamSpec *pspec) {
  IOModel *self = IO_MODEL(object);

  switch (property_id) {
  case PROP_CLOCK_UNIX_SECONDS:
    g_value_set_int64(value, self->clock_unix_seconds);
    break;
  case PROP_LANGUAGE_TEXT:
    g_value_set_string(value, self->language_text);
    break;
  case PROP_MEMORY_USED:
    g_value_set_double(value, self->memory_used);
    break;
  case PROP_MEMORY_TOTAL:
    g_value_set_double(value, self->memory_total);
    break;
  case PROP_WEATHER_TEXT:
    g_value_set_string(value, self->weather_text);
    break;
  case PROP_NETWORK_NAME:
    g_value_set_string(value, self->network_name);
    break;
  case PROP_DOWNLOAD_BYTES_PER_SEC:
    g_value_set_uint64(value, self->download_bytes_per_sec);
    break;
  case PROP_UPLOAD_BYTES_PER_SEC:
    g_value_set_uint64(value, self->upload_bytes_per_sec);
    break;
  case PROP_SOUND_VOLUME:
    g_value_set_uint(value, self->sound_volume);
    break;
  case PROP_SOUND_MUTED:
    g_value_set_boolean(value, self->sound_muted);
    break;
  case PROP_SOUND_READY:
    g_value_set_boolean(value, self->sound_ready);
    break;
  case PROP_CAPS_LOCK_ENABLED:
    g_value_set_boolean(value, self->caps_lock_enabled);
    break;
  case PROP_WEATHER_HOURLY_FORECAST:
    g_value_set_object(value, self->weather_hourly_forecast);
    break;
  case PROP_WEATHER_DAILY_FORECAST:
    g_value_set_object(value, self->weather_daily_forecast);
    break;
  case PROP_WORKSPACES: {
    GListModel *visible = NULL;
    g_object_get(self->workspaces, "visible", &visible, NULL);
    g_value_take_object(value, visible);
    break;
  }
  case PROP_CPU_CORES: {
    GListModel *cores = NULL;
    g_object_get(self->cpu, "cores", &cores, NULL);
    g_value_take_object(value, cores);
    break;
  }
  case PROP_TRAY_APPS: {
    GListModel *apps = NULL;
    g_object_get(self->tray, "apps", &apps, NULL);
    g_value_take_object(value, apps);
    break;
  }
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void io_model_finalize(GObject *object) {
  IOModel *self = IO_MODEL(object);
  g_free(self->language_text);
  g_free(self->weather_text);
  g_free(self->network_name);
  g_free(self->network_ssid);
  g_clear_object(&self->cpu);
  g_clear_object(&self->weather_hourly_forecast);
  g_clear_object(&self->weather_daily_forecast);
  g_clear_object(&self->workspaces);
  g_clear_object(&self->tray);
  G_OBJECT_CLASS(io_model_parent_class)->finalize(object);
}

static void io_model_init(IOModel *self) {
  self->clock_unix_seconds = 0;
  self->language_text = g_strdup("--");
  self->memory_used = 0.0;
  self->memory_total = 0.0;
  self->weather_text = g_strdup("--");
  self->network_name = g_strdup("--");
  self->download_bytes_per_sec = G_MAXUINT64;
  self->upload_bytes_per_sec = G_MAXUINT64;
  self->sound_volume = 0;
  self->sound_muted = false;
  self->sound_ready = false;
  self->caps_lock_enabled = false;
  self->weather_hourly_forecast =
      g_list_store_new(weather_hour_item_get_type());
  self->weather_daily_forecast = g_list_store_new(weather_day_item_get_type());
  self->network_ssid = NULL;
  self->network_strength = 0;

  self->cpu = cpu_model_new();
  self->workspaces = workspaces_model_new();
  self->tray = tray_model_new();
}

static void io_model_class_init(IOModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = io_model_get_property;
  object_class->finalize = io_model_finalize;

  properties[PROP_CLOCK_UNIX_SECONDS] = g_param_spec_int64(
      "clock-unix-seconds", NULL, NULL, 0, G_MAXINT64, 0, G_PARAM_READABLE);
  properties[PROP_LANGUAGE_TEXT] =
      g_param_spec_string("language-text", NULL, NULL, "--", G_PARAM_READABLE);
  properties[PROP_MEMORY_USED] = g_param_spec_double(
      "memory-used", NULL, NULL, 0.0, G_MAXDOUBLE, 0.0, G_PARAM_READABLE);
  properties[PROP_MEMORY_TOTAL] = g_param_spec_double(
      "memory-total", NULL, NULL, 0.0, G_MAXDOUBLE, 0.0, G_PARAM_READABLE);
  properties[PROP_WEATHER_TEXT] =
      g_param_spec_string("weather-text", NULL, NULL, "--", G_PARAM_READABLE);
  properties[PROP_NETWORK_NAME] =
      g_param_spec_string("network-name", NULL, NULL, "--", G_PARAM_READABLE);
  properties[PROP_DOWNLOAD_BYTES_PER_SEC] =
      g_param_spec_uint64("download-bytes-per-sec", NULL, NULL, 0, G_MAXUINT64,
                          G_MAXUINT64, G_PARAM_READABLE);
  properties[PROP_UPLOAD_BYTES_PER_SEC] =
      g_param_spec_uint64("upload-bytes-per-sec", NULL, NULL, 0, G_MAXUINT64,
                          G_MAXUINT64, G_PARAM_READABLE);
  properties[PROP_SOUND_VOLUME] = g_param_spec_uint(
      "sound-volume", NULL, NULL, 0, G_MAXUINT, 0, G_PARAM_READABLE);
  properties[PROP_SOUND_MUTED] =
      g_param_spec_boolean("sound-muted", NULL, NULL, false, G_PARAM_READABLE);
  properties[PROP_SOUND_READY] =
      g_param_spec_boolean("sound-ready", NULL, NULL, false, G_PARAM_READABLE);
  properties[PROP_CAPS_LOCK_ENABLED] = g_param_spec_boolean(
      "caps-lock-enabled", NULL, NULL, false, G_PARAM_READABLE);
  properties[PROP_WEATHER_HOURLY_FORECAST] =
      g_param_spec_object("weather-hourly-forecast", NULL, NULL,
                          G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_WEATHER_DAILY_FORECAST] =
      g_param_spec_object("weather-daily-forecast", NULL, NULL,
                          G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_WORKSPACES] = g_param_spec_object(
      "workspaces", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_CPU_CORES] = g_param_spec_object(
      "cpu-cores", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_TRAY_APPS] = g_param_spec_object(
      "tray-apps", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

IOModel *io_model_new(void) { return g_object_new(io_model_get_type(), NULL); }

void io_model_set_clock_unix_seconds(IOModel *self, int64_t unix_seconds) {
  self->clock_unix_seconds = unix_seconds;
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_CLOCK_UNIX_SECONDS]);
}

void io_model_set_download_bytes_per_sec(IOModel *self,
                                         uint64_t bytes_per_sec) {
  self->download_bytes_per_sec = bytes_per_sec;
  g_object_notify_by_pspec(G_OBJECT(self),
                           properties[PROP_DOWNLOAD_BYTES_PER_SEC]);
}

void io_model_set_upload_bytes_per_sec(IOModel *self, uint64_t bytes_per_sec) {
  self->upload_bytes_per_sec = bytes_per_sec;
  g_object_notify_by_pspec(G_OBJECT(self),
                           properties[PROP_UPLOAD_BYTES_PER_SEC]);
}

void io_model_set_workspaces(IOModel *self,
                             struct IO_FFIArray_HyprlandWorkspace data) {
  workspaces_model_update(self->workspaces, data);
}

void io_model_set_weather(IOModel *self,
                          struct IO_Event_IO_Weather_Body weather) {
  char buffer[100];
  snprintf(buffer, sizeof(buffer), "%.1f\xe2\x84\x83 %s", weather.temperature,
           weather_code_to_description(weather.code));
  g_free(self->weather_text);
  self->weather_text = g_strdup(buffer);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_WEATHER_TEXT]);

  g_list_store_remove_all(self->weather_hourly_forecast);
  for (size_t i = 0; i < weather.hourly_forecast.len; i++) {
    WeatherHourItem *item =
        weather_hour_item_new(weather.hourly_forecast.ptr[i]);
    g_list_store_append(self->weather_hourly_forecast, item);
    g_object_unref(item);
  }

  g_list_store_remove_all(self->weather_daily_forecast);
  for (size_t i = 0; i < weather.daily_forecast.len; i++) {
    WeatherDayItem *item = weather_day_item_new(weather.daily_forecast.ptr[i]);
    g_list_store_append(self->weather_daily_forecast, item);
    g_object_unref(item);
  }
}

void io_model_set_language(IOModel *self, const char *lang) {
  const char *text;
  if (strcmp(lang, "English (US)") == 0)
    text = "EN";
  else if (strcmp(lang, "Polish") == 0)
    text = "PL";
  else
    text = "??";
  g_free(self->language_text);
  self->language_text = g_strdup(text);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_LANGUAGE_TEXT]);
}

void io_model_set_cpu(IOModel *self, IO_FFIArray_u8 data) {
  cpu_model_update(self->cpu, data);
}

void io_model_set_memory(IOModel *self, float used, float total) {
  self->memory_used = used;
  self->memory_total = total;
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_MEMORY_USED]);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_MEMORY_TOTAL]);
}

static void refresh_network_name(IOModel *self) {
  char buffer[100];
  if (self->network_ssid)
    snprintf(buffer, sizeof(buffer), "%s (%d)%% ", self->network_ssid,
             self->network_strength);
  else
    snprintf(buffer, sizeof(buffer), "Not connected");
  g_free(self->network_name);
  self->network_name = g_strdup(buffer);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_NETWORK_NAME]);
}

void io_model_set_network_ssid(IOModel *self, const char *ssid) {
  g_free(self->network_ssid);
  self->network_ssid = g_strdup(ssid);
  refresh_network_name(self);
}

void io_model_set_network_strength(IOModel *self, uint8_t strength) {
  self->network_strength = strength;
  refresh_network_name(self);
}

void io_model_set_sound_initial(IOModel *self, uint32_t volume, bool muted) {
  self->sound_volume = volume;
  self->sound_muted = muted;
  self->sound_ready = true;
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_SOUND_VOLUME]);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_SOUND_MUTED]);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_SOUND_READY]);
}

void io_model_set_sound_volume(IOModel *self, uint32_t volume) {
  self->sound_volume = volume;
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_SOUND_VOLUME]);
}

void io_model_set_sound_muted(IOModel *self, bool muted) {
  self->sound_muted = muted;
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_SOUND_MUTED]);
}

void io_model_set_caps_lock_enabled(IOModel *self, bool enabled) {
  self->caps_lock_enabled = enabled;
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_CAPS_LOCK_ENABLED]);
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

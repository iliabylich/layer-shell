#include "ui/io_model.h"
#include "ui/cpu_model.h"
#include "ui/tray_model.h"
#include "ui/weather_helper.h"
#include "ui/workspaces_model.h"
#include <string.h>

struct _IOModel {
  GObject parent_instance;

  char *clock_text;
  char *language_text;
  double memory_used;
  double memory_total;
  char *weather_text;
  char *network_name;
  char *download_speed;
  char *upload_speed;
  CpuModel *cpu;
  WorkspacesModel *workspaces;
  TrayModel *tray;

  char *network_ssid;
  uint8_t network_strength;
};

G_DEFINE_TYPE(IOModel, io_model, G_TYPE_OBJECT)

enum {
  PROP_CLOCK_TEXT = 1,
  PROP_LANGUAGE_TEXT,
  PROP_MEMORY_USED,
  PROP_MEMORY_TOTAL,
  PROP_WEATHER_TEXT,
  PROP_NETWORK_NAME,
  PROP_DOWNLOAD_SPEED,
  PROP_UPLOAD_SPEED,
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
  case PROP_CLOCK_TEXT:
    g_value_set_string(value, self->clock_text);
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
  case PROP_DOWNLOAD_SPEED:
    g_value_set_string(value, self->download_speed);
    break;
  case PROP_UPLOAD_SPEED:
    g_value_set_string(value, self->upload_speed);
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
  g_free(self->clock_text);
  g_free(self->language_text);
  g_free(self->weather_text);
  g_free(self->network_name);
  g_free(self->download_speed);
  g_free(self->upload_speed);
  g_free(self->network_ssid);
  g_clear_object(&self->cpu);
  g_clear_object(&self->workspaces);
  g_clear_object(&self->tray);
  G_OBJECT_CLASS(io_model_parent_class)->finalize(object);
}

static void io_model_init(IOModel *self) {
  self->clock_text = g_strdup("--");
  self->language_text = g_strdup("--");
  self->memory_used = 0.0;
  self->memory_total = 0.0;
  self->weather_text = g_strdup("--");
  self->network_name = g_strdup("--");
  self->download_speed = g_strdup("??");
  self->upload_speed = g_strdup("??");
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

  properties[PROP_CLOCK_TEXT] =
      g_param_spec_string("clock-text", NULL, NULL, "--", G_PARAM_READABLE);
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
  properties[PROP_DOWNLOAD_SPEED] =
      g_param_spec_string("download-speed", NULL, NULL, "??", G_PARAM_READABLE);
  properties[PROP_UPLOAD_SPEED] =
      g_param_spec_string("upload-speed", NULL, NULL, "??", G_PARAM_READABLE);
  properties[PROP_WORKSPACES] = g_param_spec_object(
      "workspaces", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_CPU_CORES] = g_param_spec_object(
      "cpu-cores", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_TRAY_APPS] = g_param_spec_object(
      "tray-apps", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

IOModel *io_model_new(void) { return g_object_new(io_model_get_type(), NULL); }

void io_model_set_clock_text(IOModel *self, const char *text) {
  g_free(self->clock_text);
  self->clock_text = g_strdup(text);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_CLOCK_TEXT]);
}

void io_model_set_download_speed(IOModel *self, const char *text) {
  g_free(self->download_speed);
  self->download_speed = g_strdup(text);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_DOWNLOAD_SPEED]);
}

void io_model_set_upload_speed(IOModel *self, const char *text) {
  g_free(self->upload_speed);
  self->upload_speed = g_strdup(text);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_UPLOAD_SPEED]);
}

void io_model_set_workspaces(IOModel *self,
                             struct IO_FFIArray_HyprlandWorkspace data) {
  workspaces_model_update(self->workspaces, data);
}

void io_model_set_weather(IOModel *self, float temperature,
                          IO_WeatherCode code) {
  char buffer[100];
  snprintf(buffer, sizeof(buffer), "%.1f\xe2\x84\x83 %s", temperature,
           weather_code_to_description(code));
  g_free(self->weather_text);
  self->weather_text = g_strdup(buffer);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_WEATHER_TEXT]);
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

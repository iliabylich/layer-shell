#include "ui/view_models/network_model.h"
#include <stdio.h>

struct _NetworkModel {
  GObject parent_instance;

  char *ssid;
  guint8 strength;
  char *name;
  guint64 download_bytes_per_sec;
  guint64 upload_bytes_per_sec;
};

G_DEFINE_TYPE(NetworkModel, network_model, G_TYPE_OBJECT)

enum {
  PROP_SSID = 1,
  PROP_STRENGTH,
  PROP_NAME,
  PROP_DOWNLOAD_BYTES_PER_SEC,
  PROP_UPLOAD_BYTES_PER_SEC,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void refresh_name(NetworkModel *self) {
  char buffer[100];
  if (self->ssid) {
    snprintf(buffer, sizeof(buffer), "%s (%u)%% ", self->ssid, self->strength);
  } else {
    snprintf(buffer, sizeof(buffer), "Not connected");
  }
  g_free(self->name);
  self->name = g_strdup(buffer);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_NAME]);
}

static void network_model_get_property(GObject *object, guint property_id,
                                       GValue *value, GParamSpec *pspec) {
  NetworkModel *self = NETWORK_MODEL(object);
  switch (property_id) {
  case PROP_SSID:
    g_value_set_string(value, self->ssid);
    break;
  case PROP_STRENGTH:
    g_value_set_uchar(value, self->strength);
    break;
  case PROP_NAME:
    g_value_set_string(value, self->name);
    break;
  case PROP_DOWNLOAD_BYTES_PER_SEC:
    g_value_set_uint64(value, self->download_bytes_per_sec);
    break;
  case PROP_UPLOAD_BYTES_PER_SEC:
    g_value_set_uint64(value, self->upload_bytes_per_sec);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void network_model_set_property(GObject *object, guint property_id,
                                       const GValue *value, GParamSpec *pspec) {
  NetworkModel *self = NETWORK_MODEL(object);
  switch (property_id) {
  case PROP_SSID:
    g_free(self->ssid);
    self->ssid = g_value_dup_string(value);
    g_object_notify_by_pspec(object, properties[PROP_SSID]);
    refresh_name(self);
    break;
  case PROP_STRENGTH:
    self->strength = g_value_get_uchar(value);
    g_object_notify_by_pspec(object, properties[PROP_STRENGTH]);
    refresh_name(self);
    break;
  case PROP_DOWNLOAD_BYTES_PER_SEC:
    self->download_bytes_per_sec = g_value_get_uint64(value);
    g_object_notify_by_pspec(object, properties[PROP_DOWNLOAD_BYTES_PER_SEC]);
    break;
  case PROP_UPLOAD_BYTES_PER_SEC:
    self->upload_bytes_per_sec = g_value_get_uint64(value);
    g_object_notify_by_pspec(object, properties[PROP_UPLOAD_BYTES_PER_SEC]);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void network_model_finalize(GObject *object) {
  NetworkModel *self = NETWORK_MODEL(object);
  g_free(self->ssid);
  g_free(self->name);
  G_OBJECT_CLASS(network_model_parent_class)->finalize(object);
}

static void network_model_init(NetworkModel *self) {
  self->ssid = NULL;
  self->strength = 0;
  self->name = g_strdup("--");
  self->download_bytes_per_sec = G_MAXUINT64;
  self->upload_bytes_per_sec = G_MAXUINT64;
}

static void network_model_class_init(NetworkModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = network_model_get_property;
  object_class->set_property = network_model_set_property;
  object_class->finalize = network_model_finalize;

  properties[PROP_SSID] =
      g_param_spec_string("ssid", NULL, NULL, NULL,
                          G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_STRENGTH] =
      g_param_spec_uchar("strength", NULL, NULL, 0, G_MAXUINT8, 0,
                         G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_NAME] =
      g_param_spec_string("name", NULL, NULL, "--", G_PARAM_READABLE);
  properties[PROP_DOWNLOAD_BYTES_PER_SEC] = g_param_spec_uint64(
      "download_bytes_per_sec", NULL, NULL, 0, G_MAXUINT64, G_MAXUINT64,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_UPLOAD_BYTES_PER_SEC] = g_param_spec_uint64(
      "upload_bytes_per_sec", NULL, NULL, 0, G_MAXUINT64, G_MAXUINT64,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

NetworkModel *network_model_new(void) {
  return g_object_new(network_model_get_type(), NULL);
}

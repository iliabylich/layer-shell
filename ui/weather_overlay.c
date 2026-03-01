#include "ui/weather_overlay.h"
#include "ui/gobject_helper.h"
#include "ui/logger.h"
#include "ui/view_models/weather_day_item.h"
#include "ui/view_models/weather_hour_item.h"

LOGGER("WeatherOverlay", 0)

struct _WeatherOverlay {
  GtkWidget parent_instance;

  IOModel *model;
};

G_DEFINE_TYPE(WeatherOverlay, weather_overlay, BASE_OVERLAY_TYPE)

enum {
  PROP_MODEL = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static char *format_temperature(GObject *, double temperature) {
  return g_strdup_printf("%5.1f℃", temperature);
}

static char *format_hour_label(GObject *, int64_t unix_seconds) {
  if (unix_seconds <= 0)
    return g_strdup("--");

  GDateTime *dt = g_date_time_new_from_unix_local(unix_seconds);
  if (!dt)
    return g_strdup("--");

  char *formatted = g_date_time_format(dt, "%H:%M");
  g_date_time_unref(dt);
  return formatted ? formatted : g_strdup("--");
}

static char *format_day_label(GObject *, int64_t unix_seconds) {
  if (unix_seconds <= 0)
    return g_strdup("--");

  GDateTime *dt = g_date_time_new_from_unix_local(unix_seconds);
  if (!dt)
    return g_strdup("--");

  char *formatted = g_date_time_format(dt, "%b-%d");
  g_date_time_unref(dt);
  return formatted ? formatted : g_strdup("--");
}

static void toggle_requested(BaseOverlay *, gpointer data) {
  WeatherOverlay *self = WEATHER_OVERLAY(data);
  gobject_toggle_nested(G_OBJECT(self->model), "overlays", "weather");
}

static void weather_overlay_get_property(GObject *object, guint property_id,
                                         GValue *value, GParamSpec *pspec) {
  WeatherOverlay *self = WEATHER_OVERLAY(object);

  switch (property_id) {
  case PROP_MODEL:
    g_value_set_object(value, self->model);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void weather_overlay_set_property(GObject *object, guint property_id,
                                         const GValue *value,
                                         GParamSpec *pspec) {
  WeatherOverlay *self = WEATHER_OVERLAY(object);

  switch (property_id) {
  case PROP_MODEL: {
    g_set_object(&self->model, g_value_get_object(value));
    break;
  }
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void weather_overlay_init(WeatherOverlay *self) {
  LOG("init");
  self->model = NULL;
  gtk_widget_init_template(GTK_WIDGET(self));
  g_signal_connect(self, "toggle-requested", G_CALLBACK(toggle_requested),
                   self);
}

static void weather_overlay_dispose(GObject *object) {
  LOG("dispose");
  gtk_widget_dispose_template(GTK_WIDGET(object), weather_overlay_get_type());
  G_OBJECT_CLASS(weather_overlay_parent_class)->dispose(object);
}

static void weather_overlay_finalize(GObject *object) {
  WeatherOverlay *self = WEATHER_OVERLAY(object);
  g_clear_object(&self->model);
  G_OBJECT_CLASS(weather_overlay_parent_class)->finalize(object);
}

static void weather_overlay_class_init(WeatherOverlayClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = weather_overlay_get_property;
  object_class->set_property = weather_overlay_set_property;
  object_class->dispose = weather_overlay_dispose;
  object_class->finalize = weather_overlay_finalize;

  properties[PROP_MODEL] =
      g_param_spec_object("model", NULL, NULL, io_model_get_type(),
                          G_PARAM_READWRITE | G_PARAM_CONSTRUCT_ONLY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  g_type_ensure(weather_hour_item_get_type());
  g_type_ensure(weather_day_item_get_type());

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(
      widget_class, "/layer-shell/weather_overlay.ui");
  gtk_widget_class_bind_template_callback(widget_class, format_temperature);
  gtk_widget_class_bind_template_callback(widget_class, format_hour_label);
  gtk_widget_class_bind_template_callback(widget_class, format_day_label);
}

GtkWidget *weather_overlay_new(GtkApplication *app, IOModel *model) {
  return g_object_new(weather_overlay_get_type(), "application", app, "model",
                      model, NULL);
}

#include "ui/weather_overlay.h"
#include "ui/gobject_helper.h"
#include "ui/logger.h"
#include "ui/view_models/weather_day_item.h"
#include "ui/view_models/weather_hour_item.h"

LOGGER("WeatherOverlay", 0)

struct _WeatherOverlay {
  GtkWidget parent_instance;
};

G_DEFINE_TYPE(WeatherOverlay, weather_overlay, BASE_OVERLAY_TYPE)

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
  gobject_toggle_nested(G_OBJECT(base_overlay_get_model(BASE_OVERLAY(self))),
                        "overlays", "weather");
}

static void weather_overlay_init(WeatherOverlay *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));
  g_signal_connect(self, "toggle-requested", G_CALLBACK(toggle_requested),
                   self);
}

static void weather_overlay_dispose(GObject *object) {
  LOG("dispose");
  gtk_widget_dispose_template(GTK_WIDGET(object), weather_overlay_get_type());
  G_OBJECT_CLASS(weather_overlay_parent_class)->dispose(object);
}

static void weather_overlay_class_init(WeatherOverlayClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = weather_overlay_dispose;

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

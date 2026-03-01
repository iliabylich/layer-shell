#include "ui/weather_hour_item.h"
#include "ui/weather_helper.h"

struct _WeatherHourItem {
  GObject parent_instance;

  char *hour;
  double temperature;
  char *icon;
  char *description;
};

G_DEFINE_TYPE(WeatherHourItem, weather_hour_item, G_TYPE_OBJECT)

enum {
  PROP_HOUR = 1,
  PROP_TEMPERATURE,
  PROP_ICON,
  PROP_DESCRIPTION,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void weather_hour_item_get_property(GObject *object, guint property_id,
                                           GValue *value, GParamSpec *pspec) {
  WeatherHourItem *self = WEATHER_HOUR_ITEM(object);
  switch (property_id) {
  case PROP_HOUR:
    g_value_set_string(value, self->hour);
    break;
  case PROP_TEMPERATURE:
    g_value_set_double(value, self->temperature);
    break;
  case PROP_ICON:
    g_value_set_string(value, self->icon);
    break;
  case PROP_DESCRIPTION:
    g_value_set_string(value, self->description);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void weather_hour_item_finalize(GObject *object) {
  WeatherHourItem *self = WEATHER_HOUR_ITEM(object);
  g_free(self->hour);
  g_free(self->icon);
  g_free(self->description);
  G_OBJECT_CLASS(weather_hour_item_parent_class)->finalize(object);
}

static void weather_hour_item_init(WeatherHourItem *self) {
  self->hour = g_strdup("??");
  self->temperature = 0.0;
  self->icon = g_strdup("");
  self->description = g_strdup("Unknown");
}

static void weather_hour_item_class_init(WeatherHourItemClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = weather_hour_item_get_property;
  object_class->finalize = weather_hour_item_finalize;

  properties[PROP_HOUR] =
      g_param_spec_string("hour", NULL, NULL, "??", G_PARAM_READABLE);
  properties[PROP_TEMPERATURE] =
      g_param_spec_double("temperature", NULL, NULL, -G_MAXDOUBLE, G_MAXDOUBLE,
                          0.0, G_PARAM_READABLE);
  properties[PROP_ICON] =
      g_param_spec_string("icon", NULL, NULL, "", G_PARAM_READABLE);
  properties[PROP_DESCRIPTION] = g_param_spec_string(
      "description", NULL, NULL, "Unknown", G_PARAM_READABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

WeatherHourItem *weather_hour_item_new(IO_WeatherOnHour weather_on_hour) {
  WeatherHourItem *item = g_object_new(weather_hour_item_get_type(), NULL);
  g_free(item->hour);
  item->hour = g_strdup(weather_on_hour.hour);
  item->temperature = weather_on_hour.temperature;

  g_free(item->icon);
  item->icon = g_strdup(weather_code_to_icon(weather_on_hour.code));

  g_free(item->description);
  item->description =
      g_strdup(weather_code_to_description(weather_on_hour.code));
  return item;
}

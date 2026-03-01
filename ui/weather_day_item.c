#include "ui/weather_day_item.h"
#include "ui/weather_helper.h"

struct _WeatherDayItem {
  GObject parent_instance;

  char *day;
  double temperature_min;
  double temperature_max;
  char *icon;
  char *description;
};

G_DEFINE_TYPE(WeatherDayItem, weather_day_item, G_TYPE_OBJECT)

enum {
  PROP_DAY = 1,
  PROP_TEMPERATURE_MIN,
  PROP_TEMPERATURE_MAX,
  PROP_ICON,
  PROP_DESCRIPTION,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void weather_day_item_get_property(GObject *object, guint property_id,
                                          GValue *value, GParamSpec *pspec) {
  WeatherDayItem *self = WEATHER_DAY_ITEM(object);
  switch (property_id) {
  case PROP_DAY:
    g_value_set_string(value, self->day);
    break;
  case PROP_TEMPERATURE_MIN:
    g_value_set_double(value, self->temperature_min);
    break;
  case PROP_TEMPERATURE_MAX:
    g_value_set_double(value, self->temperature_max);
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

static void weather_day_item_finalize(GObject *object) {
  WeatherDayItem *self = WEATHER_DAY_ITEM(object);
  g_free(self->day);
  g_free(self->icon);
  g_free(self->description);
  G_OBJECT_CLASS(weather_day_item_parent_class)->finalize(object);
}

static void weather_day_item_init(WeatherDayItem *self) {
  self->day = g_strdup("??");
  self->temperature_min = 0.0;
  self->temperature_max = 0.0;
  self->icon = g_strdup("");
  self->description = g_strdup("Unknown");
}

static void weather_day_item_class_init(WeatherDayItemClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = weather_day_item_get_property;
  object_class->finalize = weather_day_item_finalize;

  properties[PROP_DAY] =
      g_param_spec_string("day", NULL, NULL, "??", G_PARAM_READABLE);
  properties[PROP_TEMPERATURE_MIN] =
      g_param_spec_double("temperature-min", NULL, NULL, -G_MAXDOUBLE,
                          G_MAXDOUBLE, 0.0, G_PARAM_READABLE);
  properties[PROP_TEMPERATURE_MAX] =
      g_param_spec_double("temperature-max", NULL, NULL, -G_MAXDOUBLE,
                          G_MAXDOUBLE, 0.0, G_PARAM_READABLE);
  properties[PROP_ICON] =
      g_param_spec_string("icon", NULL, NULL, "", G_PARAM_READABLE);
  properties[PROP_DESCRIPTION] = g_param_spec_string(
      "description", NULL, NULL, "Unknown", G_PARAM_READABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

WeatherDayItem *weather_day_item_new(IO_WeatherOnDay weather_on_day) {
  WeatherDayItem *item = g_object_new(weather_day_item_get_type(), NULL);
  g_free(item->day);
  item->day = g_strdup(weather_on_day.day);
  item->temperature_min = weather_on_day.temperature_min;
  item->temperature_max = weather_on_day.temperature_max;

  g_free(item->icon);
  item->icon = g_strdup(weather_code_to_icon(weather_on_day.code));

  g_free(item->description);
  item->description =
      g_strdup(weather_code_to_description(weather_on_day.code));
  return item;
}

#ifndef ICONS_H
#define ICONS_H

#include "gio/gio.h"

typedef enum {
  FOGGY_ICON,
  QUESTION_MARK_ICON,
  SUNNY_ICON,
  PARTLY_CLOUDY_ICON,
  RAINY_ICON,
  THUNDERSTORM_ICON,
  POWER_ICON,
  SNOWY_ICON,
  WIFI_ICON,
} icon_t;

void init_icons();
GIcon *get_icon(icon_t icon_name);

#endif // ICONS_H

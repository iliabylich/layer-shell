#ifndef ICONS_H
#define ICONS_H

#include "bindings.h"
#include "gio/gio.h"
#include "glib.h"
#include <stdio.h>
#include <time.h>

GIcon *FOGGY_ICON;
GIcon *QUESTION_MARK_ICON;
GIcon *SUNNY_ICON;
GIcon *PARTLY_CLOUDY_ICON;
GIcon *RAINY_ICON;
GIcon *THUNDERSTORM_ICON;
GIcon *POWER_ICON;
GIcon *SNOWY_ICON;
GIcon *WIFI_ICON;

GIcon *load_texture(LAYER_SHELL_IO_CBytes from) {
  GBytes *bytes = g_bytes_new_static(from.content, from.len);
  GError *err = NULL;
  return g_bytes_icon_new(bytes);
}

void init_icons() {
  FOGGY_ICON = load_texture(FOGGY_ICON_BYTES);
  QUESTION_MARK_ICON = load_texture(QUESTION_MARK_ICON_BYTES);
  SUNNY_ICON = load_texture(SUNNY_ICON_BYTES);
  PARTLY_CLOUDY_ICON = load_texture(PARTLY_CLOUDY_ICON_BYTES);
  RAINY_ICON = load_texture(RAINY_ICON_BYTES);
  THUNDERSTORM_ICON = load_texture(THUNDERSTORM_ICON_BYTES);
  POWER_ICON = load_texture(POWER_ICON_BYTES);
  SNOWY_ICON = load_texture(SNOWY_ICON_BYTES);
  WIFI_ICON = load_texture(WIFI_ICON_BYTES);

  printf("Finished loading icons...\n");
}

#endif // ICONS_H

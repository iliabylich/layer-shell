#include "icons.h"
#include "bindings.h"
#include "glib.h"
#include <stdio.h>
#include <time.h>

GIcon *foggy_icon;
GIcon *question_mark_icon;
GIcon *sunny_icon;
GIcon *partly_cloudy_icon;
GIcon *rainy_icon;
GIcon *thunderstorm_icon;
GIcon *power_icon;
GIcon *snowy_icon;
GIcon *wifi_icon;

GIcon *load_texture(LAYER_SHELL_IO_CBytes from) {
  GBytes *bytes = g_bytes_new_static(from.content, from.len);
  GError *err = NULL;
  return g_bytes_icon_new(bytes);
}

void init_icons(void) {
  foggy_icon = load_texture(FOGGY_ICON_BYTES);
  question_mark_icon = load_texture(QUESTION_MARK_ICON_BYTES);
  sunny_icon = load_texture(SUNNY_ICON_BYTES);
  partly_cloudy_icon = load_texture(PARTLY_CLOUDY_ICON_BYTES);
  rainy_icon = load_texture(RAINY_ICON_BYTES);
  thunderstorm_icon = load_texture(THUNDERSTORM_ICON_BYTES);
  power_icon = load_texture(POWER_ICON_BYTES);
  snowy_icon = load_texture(SNOWY_ICON_BYTES);
  wifi_icon = load_texture(WIFI_ICON_BYTES);

  printf("Finished loading icons...\n");
}

GIcon *get_icon(icon_t icon_name) {
  switch (icon_name) {
  case FOGGY_ICON:
    return foggy_icon;
  case QUESTION_MARK_ICON:
    return question_mark_icon;
  case SUNNY_ICON:
    return sunny_icon;
  case PARTLY_CLOUDY_ICON:
    return partly_cloudy_icon;
  case RAINY_ICON:
    return rainy_icon;
  case THUNDERSTORM_ICON:
    return foggy_icon;
  case POWER_ICON:
    return power_icon;
  case SNOWY_ICON:
    return snowy_icon;
  case WIFI_ICON:
    return wifi_icon;
  }
}

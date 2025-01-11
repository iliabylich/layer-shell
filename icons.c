#include "icons.h"
#include "bindings.h"
#include "gio/gio.h"
#include <glib.h>
#include <stdio.h>

static GIcon *foggy_icon;
GIcon *get_foggy_icon() { return foggy_icon; }

static GIcon *sunny_icon;
GIcon *get_sunny_icon() { return sunny_icon; }

static GIcon *partly_cloudy_icon;
GIcon *get_partly_cloudy_icon() { return partly_cloudy_icon; }

static GIcon *rainy_icon;
GIcon *get_rainy_icon() { return rainy_icon; }

static GIcon *thunderstorm_icon;
GIcon *get_thunderstorm_icon() { return thunderstorm_icon; }

static GIcon *snowy_icon;
GIcon *get_snowy_icon() { return snowy_icon; }

static GIcon *power_icon;
GIcon *get_power_icon() { return power_icon; }

static GIcon *question_mark_icon;
GIcon *get_question_mark_icon() { return question_mark_icon; }

static GIcon *wifi_icon;
GIcon *get_wifi_icon() { return wifi_icon; }

static GIcon *load_texture(LAYER_SHELL_IO_CBytes from) {
  GBytes *bytes = g_bytes_new_static(from.content, from.len);
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

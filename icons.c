#include "icons.h"
#include "gio/gio.h"
#include <glib.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

static const uint8_t FOGGY_ICON_BYTES[] = {
#embed "icons/foggy.png" if_empty('-')
    , 0};
static GIcon *foggy_icon;
GIcon *get_foggy_icon() { return foggy_icon; }

static const uint8_t SUNNY_ICON_BYTES[] = {
#embed "icons/sunny.png" if_empty('-')
    , 0};
static GIcon *sunny_icon;
GIcon *get_sunny_icon() { return sunny_icon; }

static const uint8_t PARTLY_CLOUDY_ICON_BYTES[] = {
#embed "icons/partly_cloudy.png" if_empty('-')
    , 0};
static GIcon *partly_cloudy_icon;
GIcon *get_partly_cloudy_icon() { return partly_cloudy_icon; }

static const uint8_t RAINY_ICON_BYTES[] = {
#embed "icons/rainy.png" if_empty('-')
    , 0};
static GIcon *rainy_icon;
GIcon *get_rainy_icon() { return rainy_icon; }

static const uint8_t THUNDERSTORM_ICON_BYTES[] = {
#embed "icons/thunderstorm.png" if_empty('-')
    , 0};
static GIcon *thunderstorm_icon;
GIcon *get_thunderstorm_icon() { return thunderstorm_icon; }

static const uint8_t SNOWY_ICON_BYTES[] = {
#embed "icons/snowy.png" if_empty('-')
    , 0};
static GIcon *snowy_icon;
GIcon *get_snowy_icon() { return snowy_icon; }

static const uint8_t POWER_ICON_BYTES[] = {
#embed "icons/power.png" if_empty('-')
    , 0};
static GIcon *power_icon;
GIcon *get_power_icon() { return power_icon; }

static const uint8_t QUESTION_MARK_ICON_BYTES[] = {
#embed "icons/question_mark.png" if_empty('-')
    , 0};
static GIcon *question_mark_icon;
GIcon *get_question_mark_icon() { return question_mark_icon; }

static const uint8_t WIFI_ICON_BYTES[] = {
#embed "icons/wifi.png" if_empty('-')
    , 0};
static GIcon *wifi_icon;
GIcon *get_wifi_icon() { return wifi_icon; }

static GIcon *load_texture(const uint8_t *icon, size_t len) {
  GBytes *bytes = g_bytes_new_static(icon, len);
  return g_bytes_icon_new(bytes);
}

void init_icons(void) {
  foggy_icon = load_texture(FOGGY_ICON_BYTES, sizeof(FOGGY_ICON_BYTES));
  question_mark_icon = load_texture(QUESTION_MARK_ICON_BYTES, sizeof(QUESTION_MARK_ICON_BYTES));
  sunny_icon = load_texture(SUNNY_ICON_BYTES, sizeof(SUNNY_ICON_BYTES));
  partly_cloudy_icon = load_texture(PARTLY_CLOUDY_ICON_BYTES, sizeof(PARTLY_CLOUDY_ICON_BYTES));
  rainy_icon = load_texture(RAINY_ICON_BYTES, sizeof(RAINY_ICON_BYTES));
  thunderstorm_icon = load_texture(THUNDERSTORM_ICON_BYTES, sizeof(THUNDERSTORM_ICON_BYTES));
  power_icon = load_texture(POWER_ICON_BYTES, sizeof(POWER_ICON_BYTES));
  snowy_icon = load_texture(SNOWY_ICON_BYTES, sizeof(SNOWY_ICON_BYTES));
  wifi_icon = load_texture(WIFI_ICON_BYTES, sizeof(WIFI_ICON_BYTES));

  printf("Finished loading icons...\n");
}

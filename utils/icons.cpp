#include "include/utils/icons.hpp"

Glib::RefPtr<const Gio::Icon> init_icon(const char *data, size_t size) {
  GBytes *g_bytes = g_bytes_new_static(data, size);
  GIcon *icon_ptr = g_bytes_icon_new(g_bytes);
  auto variant_ptr = g_icon_serialize(icon_ptr);
  auto variant = Glib::wrap(variant_ptr, true);
  return Gio::Icon::deserialize(variant);
}

static const char FOGGY[] = {
#embed "../icons/foggy.png" if_empty('-')
    , 0};

static const char SUNNY[] = {
#embed "../icons/sunny.png" if_empty('-')
    , 0};

static const char PARTLY_CLOUDY[] = {
#embed "../icons/partly_cloudy.png" if_empty('-')
    , 0};

static const char RAINY[] = {
#embed "../icons/rainy.png" if_empty('-')
    , 0};

static const char THUNDERSTORM[] = {
#embed "../icons/thunderstorm.png" if_empty('-')
    , 0};

static const char SNOWY[] = {
#embed "../icons/snowy.png" if_empty('-')
    , 0};

static const char POWER[] = {
#embed "../icons/power.png" if_empty('-')
    , 0};

static const char QUESTION_MARK[] = {
#embed "../icons/question_mark.png" if_empty('-')
    , 0};

static const char WIFI[] = {
#embed "../icons/wifi.png" if_empty('-')
    , 0};

static const char DOWNLOAD_SPEED[] = {
#embed "../icons/download.png" if_empty('-')
    , 0};

static const char UPLOAD_SPEED[] = {
#embed "../icons/upload.png" if_empty('-')
    , 0};

static const char CHANGE_THEME[] = {
#embed "../icons/change_theme.png" if_empty('-')
    , 0};

namespace utils {

using IconPtrT = Glib::RefPtr<const Gio::Icon>;

void Icons::init() {
  foggy = init_icon(FOGGY, sizeof(FOGGY));
  sunny = init_icon(SUNNY, sizeof(SUNNY));
  partly_cloudy = init_icon(PARTLY_CLOUDY, sizeof(PARTLY_CLOUDY));
  rainy = init_icon(RAINY, sizeof(RAINY));
  thunderstorm = init_icon(THUNDERSTORM, sizeof(THUNDERSTORM));
  snowy = init_icon(SNOWY, sizeof(SNOWY));
  power = init_icon(POWER, sizeof(POWER));
  question_mark = init_icon(QUESTION_MARK, sizeof(QUESTION_MARK));
  wifi = init_icon(WIFI, sizeof(WIFI));
  download_speed = init_icon(DOWNLOAD_SPEED, sizeof(DOWNLOAD_SPEED));
  upload_speed = init_icon(UPLOAD_SPEED, sizeof(UPLOAD_SPEED));
  change_theme = init_icon(CHANGE_THEME, sizeof(CHANGE_THEME));
}

IconPtrT Icons::foggy;
IconPtrT &Icons::foggy_icon() { return foggy; }

IconPtrT Icons::sunny;
IconPtrT &Icons::sunny_icon() { return sunny; }

IconPtrT Icons::partly_cloudy;
IconPtrT &Icons::partly_cloudy_icon() { return partly_cloudy; }

IconPtrT Icons::rainy;
IconPtrT &Icons::rainy_icon() { return rainy; }

IconPtrT Icons::thunderstorm;
IconPtrT &Icons::thunderstorm_icon() { return thunderstorm; }

IconPtrT Icons::snowy;
IconPtrT &Icons::snowy_icon() { return snowy; }

IconPtrT Icons::power;
IconPtrT &Icons::power_icon() { return power; }

IconPtrT Icons::question_mark;
IconPtrT &Icons::question_mark_icon() { return question_mark; }

IconPtrT Icons::wifi;
IconPtrT &Icons::wifi_icon() { return wifi; }

IconPtrT Icons::download_speed;
IconPtrT &Icons::download_speed_icon() { return download_speed; }

IconPtrT Icons::upload_speed;
IconPtrT &Icons::upload_speed_icon() { return upload_speed; }

IconPtrT Icons::change_theme;
IconPtrT &Icons::change_theme_icon() { return change_theme; }

} // namespace utils

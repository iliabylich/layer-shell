#ifndef ICONS_H
#define ICONS_H

#include <gio/gio.h>

void init_icons();

GIcon *get_foggy_icon();
GIcon *get_sunny_icon();
GIcon *get_partly_cloudy_icon();
GIcon *get_rainy_icon();
GIcon *get_thunderstorm_icon();
GIcon *get_snowy_icon();
GIcon *get_power_icon();
GIcon *get_question_mark_icon();
GIcon *get_wifi_icon();

#endif // ICONS_H

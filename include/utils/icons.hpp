#pragma once

#include <gtkmm.h>

namespace utils {

class Icons {
public:
  static void init();

  static Glib::RefPtr<const Gio::Icon> &foggy_icon();
  static Glib::RefPtr<const Gio::Icon> &sunny_icon();
  static Glib::RefPtr<const Gio::Icon> &partly_cloudy_icon();
  static Glib::RefPtr<const Gio::Icon> &rainy_icon();
  static Glib::RefPtr<const Gio::Icon> &thunderstorm_icon();
  static Glib::RefPtr<const Gio::Icon> &snowy_icon();
  static Glib::RefPtr<const Gio::Icon> &power_icon();
  static Glib::RefPtr<const Gio::Icon> &question_mark_icon();
  static Glib::RefPtr<const Gio::Icon> &wifi_icon();
  static Glib::RefPtr<const Gio::Icon> &download_speed_icon();
  static Glib::RefPtr<const Gio::Icon> &upload_speed_icon();
  static Glib::RefPtr<const Gio::Icon> &change_theme_icon();

private:
  static Glib::RefPtr<const Gio::Icon> foggy;
  static Glib::RefPtr<const Gio::Icon> sunny;
  static Glib::RefPtr<const Gio::Icon> partly_cloudy;
  static Glib::RefPtr<const Gio::Icon> rainy;
  static Glib::RefPtr<const Gio::Icon> thunderstorm;
  static Glib::RefPtr<const Gio::Icon> snowy;
  static Glib::RefPtr<const Gio::Icon> power;
  static Glib::RefPtr<const Gio::Icon> question_mark;
  static Glib::RefPtr<const Gio::Icon> wifi;
  static Glib::RefPtr<const Gio::Icon> download_speed;
  static Glib::RefPtr<const Gio::Icon> upload_speed;
  static Glib::RefPtr<const Gio::Icon> change_theme;
};

} // namespace utils

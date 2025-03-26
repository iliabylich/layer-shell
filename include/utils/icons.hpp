#pragma once

#include <gtkmm.h>

namespace utils {

class Icons {
public:
  static void init();

#define X(name) static Glib::RefPtr<const Gio::Icon> name;
#include "include/utils/all-icons.hpp"
#undef X
};

} // namespace utils

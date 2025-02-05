#pragma once

#include <string>

namespace utils {

class Css {
public:
  static void load();

protected:
  static std::string main_css();
  static std::string theme_css();
};

} // namespace utils

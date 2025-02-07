#pragma once

#include "include/utils/widget-helper.hpp"
#include <gtkmm.h>

namespace widgets {

class HTop : public Gtk::Button, public utils::WidgetHelper<HTop> {
public:
  HTop();
  void activate();
};

} // namespace widgets

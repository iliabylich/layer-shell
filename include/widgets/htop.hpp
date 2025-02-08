#pragma once

#include <gtkmm.h>

namespace widgets {

class HTop : public Gtk::Button {
public:
  HTop();
  void activate(void *subscriptions);
};

} // namespace widgets

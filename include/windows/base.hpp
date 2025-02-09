#pragma once

#include <gtkmm.h>

namespace windows {

class Base : public Gtk::Window {
public:
  void toggle_on_escape();
  void toggle();
};

} // namespace windows

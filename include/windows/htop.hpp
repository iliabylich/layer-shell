#pragma once

#include "include/utils/window-helper.hpp"
#include <gtkmm.h>

namespace windows {

class HTop : public Gtk::Window, public utils::WindowHelper<HTop> {
public:
  HTop();
  void activate(const Glib::RefPtr<Gtk::Application> &app);

private:
  Gtk::Widget *terminal;
};

} // namespace windows

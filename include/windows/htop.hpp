#pragma once

#include "include/windows/base.hpp"

namespace windows {

class HTop : public Base {
public:
  HTop(const Glib::RefPtr<Gtk::Application> &app, void *ctx);
  static HTop *get();

private:
  Gtk::Widget *terminal;
};

} // namespace windows

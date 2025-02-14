#pragma once

#include "include/windows/base.hpp"

namespace windows {

class Ping : public Base {
public:
  Ping(const Glib::RefPtr<Gtk::Application> &app, void *ctx);
  static Ping *get();

private:
  Gtk::Widget *terminal;
};

} // namespace windows

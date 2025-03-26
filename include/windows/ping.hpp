#pragma once

#include "include/windows/base.hpp"
#include "src/bindings.hpp"

namespace windows {

class Ping : public Base {
public:
  Ping(const Glib::RefPtr<Gtk::Application> &app, io::Ctx *ctx);
  static Ping *get();

private:
  Gtk::Widget *terminal;
};

} // namespace windows

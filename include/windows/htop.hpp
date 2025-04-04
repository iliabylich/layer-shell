#pragma once

#include "include/windows/base.hpp"
#include "src/bindings.hpp"

namespace windows {

class HTop : public Base {
public:
  HTop(const Glib::RefPtr<Gtk::Application> &app, io::Ctx *ctx);
  static HTop *get();

private:
  Gtk::Widget *terminal;
};

} // namespace windows

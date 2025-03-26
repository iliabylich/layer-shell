#pragma once

#include "src/bindings.hpp"
#include <gtkmm.h>

namespace widgets {

class ChangeTheme : public Gtk::Button {
public:
  ChangeTheme(io::Ctx *ctx);

private:
  Gtk::Image image;
};

} // namespace widgets

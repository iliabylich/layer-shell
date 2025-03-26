#pragma once

#include "bindings.hpp"
#include <gtkmm.h>

namespace widgets {

class Session : public Gtk::Button {
public:
  Session(io::Ctx *ctx);

private:
  Gtk::Image image;
};

} // namespace widgets

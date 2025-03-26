#pragma once

#include "bindings.hpp"
#include <gtkmm.h>

namespace widgets {

class HTop : public Gtk::Button {
public:
  HTop(io::Ctx *ctx);
};

} // namespace widgets

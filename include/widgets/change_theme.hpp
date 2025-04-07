#pragma once

#include "src/bindings.hpp"
#include <gtkmm.h>

namespace widgets {

class ChangeTheme : public Gtk::Button {
public:
  ChangeTheme(io::UiCtx *ui_ctx);

private:
  Gtk::Image image;
};

} // namespace widgets

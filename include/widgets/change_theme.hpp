#pragma once

#include <gtkmm.h>

namespace widgets {

class ChangeTheme : public Gtk::Button {
public:
  ChangeTheme(void *ctx);

private:
  Gtk::Image image;
};

} // namespace widgets

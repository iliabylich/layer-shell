#pragma once

#include <gtkmm.h>

namespace widgets {

class Session : public Gtk::Button {
public:
  Session(void *ctx);

private:
  Gtk::Image image;
};

} // namespace widgets

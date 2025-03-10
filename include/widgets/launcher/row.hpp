#pragma once

#include "bindings.hpp"
#include <gtkmm.h>

namespace widgets {
namespace launcher {

class Row : public Gtk::Box {
public:
  Row();
  void update(layer_shell_io::App app);

private:
  Gtk::Image image;
  Gtk::Label label;
};

} // namespace launcher
} // namespace widgets

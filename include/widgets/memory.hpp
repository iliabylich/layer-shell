#pragma once

#include "bindings.hpp"
#include "include/utils/subscription.hpp"
#include <gtkmm.h>

namespace widgets {

class Memory : public Gtk::Button, public utils::Subscription<Memory> {
public:
  Memory();
  void activate();
  void on_io_event(const layer_shell_io::Event *event);

private:
  Gtk::Label label;
};

} // namespace widgets

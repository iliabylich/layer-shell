#pragma once

#include "include/utils/subscription.hpp"
#include <gtkmm.h>

namespace widgets {

class CPU : public Gtk::Box, public utils::Subscription<CPU> {
public:
  CPU();
  void activate();
  void on_io_event(const layer_shell_io::Event *event);

private:
  std::vector<Gtk::Label> labels;
};

} // namespace widgets

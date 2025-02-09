#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Memory : public Gtk::Button, public utils::Subscriber {
public:
  Memory(void *ctx);
  void on_io_event(layer_shell_io::Event::Memory_Body data) override;
};

} // namespace widgets

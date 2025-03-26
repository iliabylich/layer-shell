#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Memory : public Gtk::Button, public utils::Subscriber {
public:
  Memory(io::Ctx *ctx);
  void on_io_event(io::Event::Memory_Body data) override;
};

} // namespace widgets

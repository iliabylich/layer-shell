#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class CPU : public Gtk::Box, public utils::Subscriber {
public:
  CPU(void *ctx);
  void on_io_event(layer_shell_io::Event::CpuUsage_Body data) override;

private:
  std::vector<Gtk::Label> labels;
};

} // namespace widgets

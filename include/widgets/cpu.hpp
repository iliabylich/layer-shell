#pragma once

#include "include/utils/subscriber.hpp"
#include "include/widgets/cpu/label.hpp"
#include <gtkmm.h>

namespace widgets {

class CPU : public Gtk::Box, public utils::Subscriber {
public:
  CPU(void *ctx);
  void on_io_event(io::Event::CpuUsage_Body data) override;

private:
  std::vector<cpu::Label> labels;
};

} // namespace widgets

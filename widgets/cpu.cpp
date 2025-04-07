#include "include/widgets/cpu.hpp"

namespace widgets {

size_t cpu_count = 12;

CPU::CPU(io::UiCtx *ui_ctx)
    : Gtk::Box(), utils::Subscriber(ui_ctx), labels(cpu_count) {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(3);
  set_css_classes({"widget", "cpu", "padded"});
  set_name("CPU");

  for (auto &label : labels) {
    append(label);
  }
}

void CPU::on_io_event(io::Event::CpuUsage_Body data) {
  for (size_t idx = 0; idx < cpu_count; idx++) {
    auto &label = labels.at(idx);
    auto load = data.usage_per_core.ptr[idx];
    label.set_load(load);
  }
}

} // namespace widgets

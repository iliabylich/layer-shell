#include "include/widgets/cpu.hpp"
#include "bindings.hpp"

namespace widgets {

CPU::CPU() : Gtk::Box() {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(3);
  set_css_classes({"widget", "cpu", "padded"});
  set_name("CPU");

  for (size_t i = 0; i < 12; i++) {
    Gtk::Label label;
    label.set_use_markup(true);
    append(label);
    labels.push_back(std::move(label));
  }
}

void CPU::activate() { subscribe_to_io_events(); }

void CPU::on_io_event(const layer_shell_io::Event *event) {
#define INDICATORS_COUNT 8
  static const char *INDICATORS[INDICATORS_COUNT] = {
      "<span color='#FFFFFF'>▁</span>", "<span color='#FFD5D5'>▂</span>",
      "<span color='#FFAAAA'>▃</span>", "<span color='#FF8080'>▄</span>",
      "<span color='#FF5555'>▅</span>", "<span color='#FF2B2B'>▆</span>",
      "<span color='#FF0000'>▇</span>", "<span color='#E60000'>█</span>",
  };

  if (event->tag == layer_shell_io::Event::Tag::CpuUsage) {
    for (size_t idx = 0; idx < 12; idx++) {
      Gtk::Label &label = labels.at(idx);
      size_t load = event->cpu_usage.usage_per_core.ptr[idx];
      size_t indicator_idx =
          (size_t)((double)load / 100.0 * (double)INDICATORS_COUNT);

      if (indicator_idx == INDICATORS_COUNT) {
        indicator_idx -= 1;
      }

      const char *markup = INDICATORS[indicator_idx];
      label.set_label(markup);
    }
  }
}

} // namespace widgets

#include "include/widgets/memory.hpp"
#include "bindings.hpp"

namespace widgets {

Memory::Memory() : Gtk::Button() {
  set_css_classes({"widget", "memory", "padded", "clickable"});
  set_name("Memory");
}

void Memory::activate() {
  signal_clicked().connect(
      []() { layer_shell_io::layer_shell_io_spawn_system_monitor(); });

  subscribe_to_io_events();
}

void Memory::on_io_event(const layer_shell_io::Event *event) {
  if (event->tag == layer_shell_io::Event::Tag::Memory) {
    char buffer[100];
    sprintf(buffer, "RAM %.1fG/%.1fG", event->memory.used, event->memory.total);
    set_label(buffer);
  }
}

} // namespace widgets

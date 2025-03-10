#include "include/widgets/memory.hpp"

namespace widgets {

Memory::Memory(void *ctx) : Gtk::Button("--"), utils::Subscriber(ctx) {
  set_css_classes({"widget", "memory", "padded", "clickable"});
  set_name("Memory");
  signal_clicked().connect(
      [ctx]() { layer_shell_io::layer_shell_io_spawn_system_monitor(ctx); });
}

void Memory::on_io_event(layer_shell_io::Event::Memory_Body data) {
  char buffer[100];
  sprintf(buffer, "RAM %.1fG/%.1fG", data.used, data.total);
  set_label(buffer);
}

} // namespace widgets

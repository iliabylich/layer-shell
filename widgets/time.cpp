#include "include/widgets/time.hpp"

namespace widgets {

Time::Time(void *ctx) : Gtk::Label("--"), utils::Subscriber(ctx) {
  set_css_classes({"widget", "clock", "padded"});
  set_name("Time");
}

void Time::on_io_event(layer_shell_io::Event::Time_Body data) {
  set_label(data.time);
}

} // namespace widgets

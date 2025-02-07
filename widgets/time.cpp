#include "include/widgets/time.hpp"
#include "bindings.hpp"

namespace widgets {

Time::Time() : Gtk::Label() {
  set_css_classes({"widget", "clock", "padded"});
  set_name("Time");

  set_label("--");
}

void Time::activate() { subscribe_to_io_events(); }

void Time::on_io_event(const layer_shell_io::Event *event) {
  if (event->tag == layer_shell_io::Event::Tag::Time) {
    set_label(event->time.time);
    set_tooltip_text(event->time.date);
  }
}

} // namespace widgets

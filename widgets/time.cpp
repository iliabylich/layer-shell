#include "include/widgets/time.hpp"
#include "bindings.hpp"

namespace widgets {

Time::Time() : Gtk::CenterBox() {
  set_css_classes({"widget", "clock", "padded"});
  set_name("Time");

  label.set_label("--");
  set_center_widget(label);
}

void Time::activate() { subscribe_to_io_events(); }

void Time::on_io_event(const layer_shell_io::Event *event) {
  if (event->tag == layer_shell_io::Event::Tag::Time) {
    label.set_label(event->time.time);
    label.set_tooltip_text(event->time.date);
  }
}

} // namespace widgets

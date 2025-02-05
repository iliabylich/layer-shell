#include "include/widgets/language.hpp"
#include "bindings.hpp"

namespace widgets {

Language::Language() : Gtk::CenterBox() {
  set_css_classes({"widget", "language", "padded"});
  set_name("Language");

  label.set_label("--");
  set_center_widget(label);
}

void Language::activate() { subscribe_to_io_events(); }

void Language::on_io_event(const layer_shell_io::Event *event) {
  if (event->tag == layer_shell_io::Event::Tag::Language) {
    std::string lang(event->language.lang);

    if (lang == "English (US)") {
      label.set_label("EN");
    } else if (lang == "Polish") {
      label.set_label("PL");
    } else {
      label.set_label("??");
    }
  }
}

} // namespace widgets

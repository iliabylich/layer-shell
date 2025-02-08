#include "include/widgets/language.hpp"
#include "bindings.hpp"

namespace widgets {

Language::Language() : Gtk::Label() {
  set_css_classes({"widget", "language", "padded"});
  set_name("Language");

  set_label("--");
}

void Language::activate(void *subscriptions) {
  subscribe_to_io_events(subscriptions);
}

void Language::on_io_event(const layer_shell_io::Event *event) {
  if (event->tag == layer_shell_io::Event::Tag::Language) {
    std::string lang(event->language.lang);

    if (lang == "English (US)") {
      set_label("EN");
    } else if (lang == "Polish") {
      set_label("PL");
    } else {
      set_label("??");
    }
  }
}

} // namespace widgets

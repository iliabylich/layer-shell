#include "include/widgets/language.hpp"

namespace widgets {

Language::Language(void *ctx) : Gtk::Label(), utils::Subscriber(ctx) {
  set_css_classes({"widget", "language", "padded"});
  set_name("Language");

  set_label("--");
}

void Language::on_language_event(layer_shell_io::Event::Language_Body data) {
  std::string lang(data.lang);

  if (lang == "English (US)") {
    set_label("EN");
  } else if (lang == "Polish") {
    set_label("PL");
  } else {
    set_label("??");
  }
}

} // namespace widgets

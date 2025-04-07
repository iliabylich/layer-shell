#include "include/widgets/language.hpp"

namespace widgets {

Language::Language(io::UiCtx *ui_ctx)
    : Gtk::Label("--"), utils::Subscriber(ui_ctx) {
  set_css_classes({"widget", "language", "padded"});
  set_name("Language");
}

void Language::on_io_event(io::Event::Language_Body data) {
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

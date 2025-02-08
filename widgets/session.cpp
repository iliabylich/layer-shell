#include "include/widgets/session.hpp"
#include "include/utils/icons.hpp"
#include "include/windows/session.hpp"

namespace widgets {

Session::Session() : Gtk::Button() {
  set_css_classes({"widget", "power", "padded", "clickable"});
  set_cursor("pointer");
  set_name("Session");

  image.set(utils::Icons::power_icon());
  set_child(image);
}

void Session::activate(void *) {
  signal_clicked().connect([]() { windows::Session::toggle(); });
}

} // namespace widgets

#include "include/widgets/htop.hpp"
#include "include/windows/htop.hpp"

namespace widgets {

HTop::HTop() : Gtk::Button() {
  set_css_classes({"widget", "terminal", "padded", "clickable"});
  set_name("HTop");
  set_label("HTop");
}

void HTop::activate(void *) {
  signal_clicked().connect([]() { windows::HTop::toggle(); });
}

} // namespace widgets

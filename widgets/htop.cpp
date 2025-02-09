#include "include/widgets/htop.hpp"
#include "include/windows/htop.hpp"

namespace widgets {

HTop::HTop(void *) : Gtk::Button() {
  set_css_classes({"widget", "terminal", "padded", "clickable"});
  set_name("HTop");
  set_label("HTop");
  signal_clicked().connect([]() { windows::HTop::get()->toggle(); });
}

} // namespace widgets

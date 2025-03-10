#include "include/widgets/htop.hpp"
#include "include/windows/htop.hpp"

namespace widgets {

HTop::HTop(void *) : Gtk::Button("HTop") {
  set_css_classes({"widget", "terminal", "padded", "clickable"});
  set_name("HTop");
  signal_clicked().connect([]() { windows::HTop::get()->toggle(); });
}

} // namespace widgets

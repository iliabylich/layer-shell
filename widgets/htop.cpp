#include "include/widgets/htop.hpp"
#include "include/windows/htop.hpp"
#include "include/windows/top-bar.hpp"

namespace widgets {

HTop::HTop() : Gtk::Button() {
  set_css_classes({"widget", "terminal", "padded", "clickable"});
  set_name("HTop");
  label.set_label("HTop");
  set_child(label);
}

void HTop::activate() {
  signal_clicked().connect([this]() {
    auto bottom_right = this->bottom_right_point(*windows::TopBar::instance());
    windows::HTop::move(bottom_right.get_x() - (float)windows::HTop::WIDTH / 2,
                        bottom_right.get_y());
    windows::HTop::toggle();
  });
}

} // namespace widgets

#include "include/widgets/workspaces.hpp"
#include "bindings.hpp"

namespace widgets {

Workspaces::Workspaces() : Gtk::Box() {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(0);
  set_css_classes({"widget", "workspaces"});
  set_name("Workspaces");

  for (size_t i = 0; i < 10; i++) {
    Gtk::Button button;
    Gtk::Label label(std::to_string(i + 1));
    button.set_child(label);
    append(button);
    buttons.push_back(std::move(button));
  }
}

void Workspaces::activate(void *subscriptions) {
  for (size_t idx = 0; idx < 10; idx++) {
    Gtk::Button &button = buttons.at(idx);
    button.signal_clicked().connect([idx]() {
      layer_shell_io::layer_shell_io_hyprland_go_to_workspace(idx);
    });
  }

  subscribe_to_io_events(subscriptions);
}

void Workspaces::on_io_event(const layer_shell_io::Event *event) {
  if (event->tag == layer_shell_io::Event::Tag::Workspaces) {
    for (size_t idx = 1; idx <= 10; idx++) {
      Gtk::Button &button = buttons.at(idx - 1);
      bool visible = false;
      for (size_t i = 0; i < event->workspaces.ids.len; i++) {
        if (event->workspaces.ids.ptr[i] == idx) {
          visible = true;
        }
      }
      button.set_visible(visible || idx <= 5);
      button.remove_css_class("active");
      button.remove_css_class("inactive");
      if (idx == event->workspaces.active_id) {
        button.add_css_class("active");
      } else {
        button.add_css_class("inactive");
      }
    }
  }
}

} // namespace widgets

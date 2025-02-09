#include "include/widgets/workspaces.hpp"

namespace widgets {

Workspaces::Workspaces(void *ctx) : Gtk::Box(), utils::Subscriber(ctx) {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(0);
  set_css_classes({"widget", "workspaces"});
  set_name("Workspaces");

  for (size_t idx = 0; idx < 10; idx++) {
    Gtk::Button button;
    Gtk::Label label(std::to_string(idx + 1));
    button.set_child(label);
    append(button);

    button.signal_clicked().connect([ctx, idx]() {
      layer_shell_io::layer_shell_io_hyprland_go_to_workspace(idx, ctx);
    });

    buttons.push_back(std::move(button));
  }
}

void Workspaces::on_io_event(layer_shell_io::Event::Workspaces_Body data) {
  for (size_t idx = 1; idx <= 10; idx++) {
    Gtk::Button &button = buttons.at(idx - 1);
    bool visible = false;
    for (size_t i = 0; i < data.ids.len; i++) {
      if (data.ids.ptr[i] == idx) {
        visible = true;
      }
    }
    button.set_visible(visible || idx <= 5);
    button.remove_css_class("active");
    button.remove_css_class("inactive");
    if (idx == data.active_id) {
      button.add_css_class("active");
    } else {
      button.add_css_class("inactive");
    }
  }
}

} // namespace widgets

#include "include/widgets/workspaces.hpp"

namespace widgets {

size_t workspaces_count = 10;

Workspaces::Workspaces(io::Ctx *ctx)
    : Gtk::Box(), utils::Subscriber(ctx), buttons(workspaces_count) {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(0);
  set_css_classes({"widget", "workspaces"});
  set_name("Workspaces");

  for (size_t idx = 0; idx < workspaces_count; idx++) {
    auto &button = buttons.at(idx);
    button.set_label(std::to_string(idx + 1));
    append(button);

    button.signal_clicked().connect(
        [ctx, idx]() { io::io_hyprland_go_to_workspace(idx, ctx); });
  }
}

void Workspaces::on_io_event(io::Event::Workspaces_Body data) {
  for (size_t idx = 1; idx <= workspaces_count; idx++) {
    auto &button = buttons.at(idx - 1);
    auto visible = false;
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

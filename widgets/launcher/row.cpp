#include "include/widgets/launcher/row.hpp"

namespace widgets {
namespace launcher {

Row::Row() : Gtk::Box() {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(0);
  set_css_classes({"row"});

  image.set_icon_size(Gtk::IconSize::LARGE);

  label.set_label("...");
  label.set_xalign(0.0);
  label.set_valign(Gtk::Align::CENTER);
  label.set_ellipsize(Pango::EllipsizeMode::END);

  append(image);
  append(label);
}

void Row::update(io::App app) {
  show();
  if (app.selected) {
    add_css_class("active");
  } else {
    remove_css_class("active");
  }

  if (app.icon.tag == io::AppIcon::Tag::IconName) {
    image.set_from_icon_name(app.icon.icon_name._0);
  } else {
    image.set(app.icon.icon_path._0);
  }
  label.set_label(app.name);
}

} // namespace launcher
} // namespace widgets

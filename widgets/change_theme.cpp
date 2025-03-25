#include "include/widgets/change_theme.hpp"
#include "bindings.hpp"
#include "include/utils/icons.hpp"

namespace widgets {

ChangeTheme::ChangeTheme(void *ctx) : Gtk::Button() {
  set_css_classes({"widget", "power", "padded", "clickable"});
  set_cursor("pointer");
  set_name("ChangeTheme");

  image.set(utils::Icons::change_theme_icon());
  set_child(image);

  signal_clicked().connect([ctx]() { io::io_change_theme(ctx); });
}

} // namespace widgets

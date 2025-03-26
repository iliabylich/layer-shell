#include "include/widgets/change_theme.hpp"
#include "include/utils/icons.hpp"
#include "src/bindings.hpp"

namespace widgets {

ChangeTheme::ChangeTheme(io::Ctx *ctx) : Gtk::Button() {
  set_css_classes({"widget", "power", "padded", "clickable"});
  set_cursor("pointer");
  set_name("ChangeTheme");

  image.set(utils::Icons::change_theme);
  set_child(image);

  signal_clicked().connect([ctx]() { io::io_change_theme(ctx); });
}

} // namespace widgets

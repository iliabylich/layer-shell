#include "include/windows/base.hpp"

namespace windows {

void Base::toggle_on_escape() {
  auto ctrl = Gtk::EventControllerKey::create();
  ctrl->signal_key_pressed().connect(
      [this](guint keyval, guint, Gdk::ModifierType) {
        if (std::string("Escape") == gdk_keyval_name(keyval)) {
          toggle();
          return true;
        }
        return false;
      },
      true);
  ctrl->set_propagation_phase(Gtk::PropagationPhase::CAPTURE);
  add_controller(ctrl);
}

void Base::toggle() {
  if (get_visible()) {
    hide();
  } else {
    show();
  }
}

} // namespace windows

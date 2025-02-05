#pragma once

#include "gtk4-layer-shell.h"
#include <gtkmm.h>

namespace utils {

template <class T> class WindowHelper {
public:
  static T *instance() {
    static T *instance = new T();
    return instance;
  }

  void toggle_on_escape() {
    auto ctrl = Gtk::EventControllerKey::create();
    ctrl->signal_key_pressed().connect(
        [](guint keyval, guint, Gdk::ModifierType) {
          if (std::string("Escape") == gdk_keyval_name(keyval)) {
            T::toggle();
          }
          return true;
        },
        true);
    ctrl->set_propagation_phase(Gtk::PropagationPhase::CAPTURE);
    static_cast<T *>(this)->add_controller(ctrl);
  }

  static void toggle() {
    if (instance()->get_visible()) {
      instance()->hide();
    } else {
      instance()->show();
    }
  }

  static void move(int x, int y) {
    auto window = instance()->gobj();
    gtk_layer_set_margin(window, GTK_LAYER_SHELL_EDGE_LEFT, x);
    gtk_layer_set_margin(window, GTK_LAYER_SHELL_EDGE_TOP, y);
  }
};

} // namespace utils

#pragma once

#include "include/windows/base.hpp"
#include "src/bindings.hpp"

namespace windows {

class Ping : public Base {
public:
  static void init(const Glib::RefPtr<Gtk::Application> &app,
                   io::UiCtx *ui_ctx);
  static Ping *get();

private:
  Ping(const Glib::RefPtr<Gtk::Application> &app, io::UiCtx *ui_ctx);

  Gtk::Widget *terminal;

  static Ping *instance;
};

} // namespace windows

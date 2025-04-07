#pragma once

#include "include/windows/base.hpp"
#include "src/bindings.hpp"

namespace windows {

class HTop : public Base {
public:
  static void init(const Glib::RefPtr<Gtk::Application> &app,
                   io::UiCtx *ui_ctx);
  static HTop *get();

private:
  HTop(const Glib::RefPtr<Gtk::Application> &app, io::UiCtx *ui_ctx);

  Gtk::Widget *terminal;

  static HTop *instance;
};

} // namespace windows

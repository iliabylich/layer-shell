#pragma once

#include "include/utils/subscriber.hpp"
#include "include/widgets/launcher/row.hpp"
#include "include/windows/base.hpp"

namespace windows {

class Launcher : public Base, public utils::Subscriber {
public:
  static void init(const Glib::RefPtr<Gtk::Application> &app,
                   io::UiCtx *ui_ctx);
  static Launcher *get();

  void toggle_and_reset();
  void on_io_event(io::Event::Launcher_Body data) override;
  void on_toggle_launcher_event() override;

private:
  Launcher(const Glib::RefPtr<Gtk::Application> &app, io::UiCtx *ui_ctx);

  std::vector<widgets::launcher::Row> rows;
  Gtk::SearchEntry input;

  static Launcher *instance;
  io::UiCtx *ui_ctx;
};

} // namespace windows

#pragma once

#include "include/utils/subscriber.hpp"
#include "include/widgets/launcher/row.hpp"
#include "include/windows/base.hpp"

namespace windows {

class Launcher : public Base, public utils::Subscriber {
public:
  Launcher(const Glib::RefPtr<Gtk::Application> &app, void *ctx);
  void toggle_and_reset();
  void on_io_event(io::Event::Launcher_Body data) override;
  void on_toggle_launcher_event() override;

  static Launcher *get();

private:
  std::vector<widgets::launcher::Row> rows;
  Gtk::SearchEntry input;
};

} // namespace windows

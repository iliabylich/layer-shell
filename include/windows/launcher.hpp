#pragma once

#include "include/utils/subscriber.hpp"
#include "include/windows/base.hpp"

namespace windows {

class Launcher : public Base, public utils::Subscriber {
public:
  Launcher(const Glib::RefPtr<Gtk::Application> &app, void *ctx);
  void toggle_and_reset();
  void on_app_list_event(layer_shell_io::Event::AppList_Body data) override;
  void on_toggle_launcher_event() override;

  static Launcher *get();

private:
  class Row : public Gtk::Box {
  public:
    Row();
    void update(layer_shell_io::App app);

  private:
    Gtk::Image image;
    Gtk::Label label;
  };

  std::vector<Row> rows;
  Gtk::SearchEntry input;
};

} // namespace windows

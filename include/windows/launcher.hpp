#pragma once

#include "bindings.hpp"
#include "include/utils/subscription.hpp"
#include "include/utils/window-helper.hpp"
#include <gtkmm.h>

namespace windows {

class Launcher : public Gtk::Window,
                 public utils::Subscription<Launcher>,
                 public utils::WindowHelper<Launcher> {
public:
  Launcher();
  void activate(const Glib::RefPtr<Gtk::Application> &app, void *subscriptions);
  void on_io_event(const layer_shell_io::Event *event);
  void toggle_and_reset();

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

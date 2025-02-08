#pragma once

#include "include/utils/subscription.hpp"
#include "include/utils/window-helper.hpp"
#include <gtkmm.h>

namespace windows {

class Session : public Gtk::Window,
                public utils::Subscription<Session>,
                public utils::WindowHelper<Session> {
public:
  Session();
  void activate(const Glib::RefPtr<Gtk::Application> &app, void *subscriptions);
  void on_io_event(const layer_shell_io::Event *event);

private:
  Gtk::Button lock;
  Gtk::Button reboot;
  Gtk::Button shutdown;
  Gtk::Button logout;
};

} // namespace windows

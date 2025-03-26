#pragma once

#include "include/utils/subscriber.hpp"
#include "include/windows/base.hpp"

namespace windows {

class Session : public Base, public utils::Subscriber {
public:
  Session(const Glib::RefPtr<Gtk::Application> &app, io::Ctx *ctx);
  void on_toggle_session_screen_event() override;
  static Session *get();

private:
  Gtk::Button lock;
  Gtk::Button reboot;
  Gtk::Button shutdown;
  Gtk::Button logout;
};

} // namespace windows

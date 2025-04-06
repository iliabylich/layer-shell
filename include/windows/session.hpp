#pragma once

#include "include/utils/subscriber.hpp"
#include "include/windows/base.hpp"

namespace windows {

class Session : public Base, public utils::Subscriber {
public:
  static void init(const Glib::RefPtr<Gtk::Application> &app, io::Ctx *ctx);
  static Session *get();
  void on_toggle_session_screen_event() override;

private:
  Session(const Glib::RefPtr<Gtk::Application> &app, io::Ctx *ctx);

  Gtk::Button lock;
  Gtk::Button reboot;
  Gtk::Button shutdown;
  Gtk::Button logout;

  static Session *instance;
};

} // namespace windows

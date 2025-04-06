#include "bindings.hpp"
#include "include/application.hpp"
#include "include/utils/css.hpp"
#include "include/utils/icons.hpp"
#include "include/windows/htop.hpp"
#include "include/windows/launcher.hpp"
#include "include/windows/ping.hpp"
#include "include/windows/session.hpp"
#include "include/windows/top-bar.hpp"
#include "include/windows/weather.hpp"
#include <gtkmm.h>
#include <iostream>

static Glib::RefPtr<Gtk::Application> app;
Glib::RefPtr<Gtk::Application> get_app() { return app; }

int main(void) {
  auto ctx = io::io_init();

  app = Gtk::Application::create("org.me.LayerShell",
                                 Gio::Application::Flags::DEFAULT_FLAGS);
  app->hold();

  app->signal_activate().connect([ctx]() {
    utils::Icons::init();

    windows::TopBar::init(app, ctx);
    windows::Session::init(app, ctx);
    windows::HTop::init(app, ctx);
    windows::Weather::init(app, ctx);
    windows::Launcher::init(app, ctx);
    windows::Ping::init(app, ctx);

    Glib::signal_timeout().connect(
        [ctx]() {
          io::io_poll_events(ctx);
          return true;
        },
        50);

    std::cout << "Finished building widgets...\n";

    io::io_spawn_thread(ctx);
  });

  app->signal_startup().connect([ctx]() {
    auto css = new utils::Css(ctx);
    css->load();
  });

  return app->run();
}

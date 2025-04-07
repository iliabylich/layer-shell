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
  auto subs = io::io_subscription_list_new();

  app = Gtk::Application::create("org.me.LayerShell",
                                 Gio::Application::Flags::DEFAULT_FLAGS);
  app->hold();

  app->signal_activate().connect([ctx, subs]() {
    utils::Icons::init();

    windows::TopBar::init(app, ctx, subs);
    windows::Session::init(app, ctx, subs);
    windows::HTop::init(app, ctx);
    windows::Weather::init(app, subs);
    windows::Launcher::init(app, ctx, subs);
    windows::Ping::init(app, ctx);

    Glib::signal_timeout().connect(
        [ctx, subs]() {
          io::io_poll_events(ctx, subs);
          return true;
        },
        50);

    std::cout << "Finished building widgets...\n";

    io::io_spawn_thread(ctx);
  });

  app->signal_startup().connect([subs]() {
    auto css = new utils::Css(subs);
    css->load();
  });

  return app->run();
}

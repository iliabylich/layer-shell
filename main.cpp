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

windows::TopBar *top_bar;
windows::TopBar *windows::TopBar::get() { return top_bar; }
windows::Session *session;
windows::Session *windows::Session::get() { return session; }
windows::HTop *htop;
windows::HTop *windows::HTop::get() { return htop; }
windows::Weather *weather;
windows::Weather *windows::Weather::get() { return weather; }
windows::Launcher *launcher;
windows::Launcher *windows::Launcher::get() { return launcher; }
windows::Ping *ping;
windows::Ping *windows::Ping::get() { return ping; }

int main(void) {
  auto ctx = layer_shell_io::layer_shell_io_init();

  app = Gtk::Application::create("org.me.LayerShell",
                                 Gio::Application::Flags::DEFAULT_FLAGS);
  app->hold();

  app->signal_activate().connect([ctx]() {
    utils::Icons::init();

    top_bar = new windows::TopBar(app, ctx);
    session = new windows::Session(app, ctx);
    htop = new windows::HTop(app, ctx);
    weather = new windows::Weather(app, ctx);
    launcher = new windows::Launcher(app, ctx);
    ping = new windows::Ping(app, ctx);

    Glib::signal_timeout().connect(
        [ctx]() {
          layer_shell_io::layer_shell_io_poll_events(ctx);
          return true;
        },
        50);

    std::cout << "Finished building widgets...\n";

    layer_shell_io::layer_shell_io_spawn_thread(ctx);
  });

  app->signal_startup().connect([]() { utils::Css::load(); });

  return app->run();
}

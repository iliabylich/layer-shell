#include "bindings.hpp"
#include "glibmm/refptr.h"
#include "gtkmm/application.h"
#include "include/application.hpp"
#include "include/utils/css.hpp"
#include "include/utils/icons.hpp"
#include "include/windows/htop.hpp"
#include "include/windows/launcher.hpp"
#include "include/windows/session.hpp"
#include "include/windows/top-bar.hpp"
#include "include/windows/weather.hpp"
#include <gtkmm.h>
#include <iostream>

static Glib::RefPtr<Gtk::Application> app;

Glib::RefPtr<Gtk::Application> get_app() { return app; }

int main(void) {
  layer_shell_io::layer_shell_io_init();

  app = Gtk::Application::create("org.me.LayerShell",
                                 Gio::Application::Flags::DEFAULT_FLAGS);
  app->hold();

  app->signal_activate().connect([]() {
    utils::Icons::init();

    auto top_bar = windows::TopBar::instance();
    auto session = windows::Session::instance();
    auto htop = windows::HTop::instance();
    auto weather = windows::Weather::instance();
    auto launcher = windows::Launcher::instance();

    Glib::signal_timeout().connect(
        []() {
          layer_shell_io::layer_shell_io_poll_events();
          return true;
        },
        50);

    top_bar->activate(app);
    session->activate(app);
    htop->activate(app);
    weather->activate(app);
    launcher->activate(app);

    std::cout << "Finished building widgets...\n";

    layer_shell_io::layer_shell_io_spawn_thread();
  });

  app->signal_startup().connect([]() { utils::Css::load(); });

  return app->run();
}

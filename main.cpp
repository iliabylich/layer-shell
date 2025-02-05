#include "bindings.hpp"
#include "include/utils/css.hpp"
#include "include/utils/icons.hpp"
#include "include/windows/htop.hpp"
#include "include/windows/launcher.hpp"
#include "include/windows/network.hpp"
#include "include/windows/session.hpp"
#include "include/windows/top-bar.hpp"
#include "include/windows/weather.hpp"
#include <gtkmm.h>
#include <iostream>

class App : public Gtk::Application {
public:
  static Glib::RefPtr<App> create() {
    auto app = Glib::make_refptr_for_instance(new App());
    app->hold();
    return app;
  }

protected:
  App()
      : Gtk::Application("org.me.LayerShell",
                         Gio::Application::Flags::DEFAULT_FLAGS) {}

private:
  sigc::connection timeout_connection;
};

int main(void) {
  layer_shell_io::layer_shell_io_init();

  auto app = App::create();

  app->signal_activate().connect([app]() {
    utils::Icons::init();

    auto top_bar = windows::TopBar::instance();
    auto session = windows::Session::instance();
    auto network = windows::Network::instance();
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
    network->activate(app);
    htop->activate(app);
    weather->activate(app);
    launcher->activate(app);

    std::cout << "Finished building widgets...\n";

    layer_shell_io::layer_shell_io_spawn_thread();
  });

  app->signal_startup().connect([]() { utils::Css::load(); });

  return app->run();
}

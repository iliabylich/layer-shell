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
  setenv("GSK_RENDERER", "cairo", true);
  auto ctx = io::io_init();
  auto io_ctx = ctx.io;
  auto ui_ctx = ctx.ui;

  app = Gtk::Application::create("org.me.LayerShell",
                                 Gio::Application::Flags::DEFAULT_FLAGS);
  app->hold();

  app->signal_activate().connect([ui_ctx, io_ctx]() {
    utils::Icons::init();

    windows::TopBar::init(app, ui_ctx);
    windows::Session::init(app, ui_ctx);
    windows::HTop::init(app, ui_ctx);
    windows::Weather::init(app, ui_ctx);
    windows::Launcher::init(app, ui_ctx);
    windows::Ping::init(app, ui_ctx);

    Glib::signal_timeout().connect(
        [ui_ctx]() {
          io::io_poll_events(ui_ctx);
          return true;
        },
        50);

    std::cout << "Finished building widgets...\n";

    io::io_spawn_thread(io_ctx);
  });

  app->signal_startup().connect([ui_ctx]() {
    auto css = new utils::Css(ui_ctx);
    css->load();
  });

  return app->run();
}

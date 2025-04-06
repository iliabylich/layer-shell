#pragma once

#include "include/widgets/change_theme.hpp"
#include "include/widgets/cpu.hpp"
#include "include/widgets/htop.hpp"
#include "include/widgets/language.hpp"
#include "include/widgets/memory.hpp"
#include "include/widgets/network.hpp"
#include "include/widgets/session.hpp"
#include "include/widgets/sound.hpp"
#include "include/widgets/time.hpp"
#include "include/widgets/tray.hpp"
#include "include/widgets/weather.hpp"
#include "include/widgets/workspaces.hpp"
#include "include/windows/base.hpp"

namespace windows {

class TopBar : public Base {
public:
  static void init(const Glib::RefPtr<Gtk::Application> &app, io::Ctx *ctx);

private:
  TopBar(const Glib::RefPtr<Gtk::Application> &app, io::Ctx *ctx);

  widgets::Workspaces workspaces;
  widgets::ChangeTheme change_theme;

  widgets::Tray tray;
  widgets::Weather weather;
  widgets::HTop htop;
  widgets::Language language;
  widgets::Sound sound;
  widgets::CPU cpu;
  widgets::Memory memory;
  widgets::Network network;
  widgets::Time time;
  widgets::Session session;

  static TopBar *instance;
};

} // namespace windows

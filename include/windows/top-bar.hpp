#pragma once

#include "include/utils/window-helper.hpp"
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
#include <gtkmm.h>

namespace windows {

class TopBar : public Gtk::Window, public utils::WindowHelper<TopBar> {
public:
  TopBar();

  void activate(const Glib::RefPtr<Gtk::Application> &app);

private:
  widgets::Workspaces workspaces;
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
};

} // namespace windows

#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Weather : public Gtk::Button, public utils::Subscriber {
public:
  Weather(io::Ctx *ctx);
  void on_io_event(io::Event::CurrentWeather_Body data) override;
};

} // namespace widgets

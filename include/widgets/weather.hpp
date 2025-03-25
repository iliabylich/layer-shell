#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Weather : public Gtk::Button, public utils::Subscriber {
public:
  Weather(void *ctx);
  void on_io_event(io::Event::CurrentWeather_Body data) override;
};

} // namespace widgets

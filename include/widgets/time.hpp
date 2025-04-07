#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Time : public Gtk::Label, public utils::Subscriber {
public:
  Time(io::Subscriptions *subs);
  void on_io_event(io::Event::Time_Body data) override;
};

} // namespace widgets

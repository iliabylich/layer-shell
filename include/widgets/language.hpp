#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Language : public Gtk::Label, public utils::Subscriber {
public:
  Language(void *ctx);
  void on_io_event(io::Event::Language_Body data) override;
};

} // namespace widgets

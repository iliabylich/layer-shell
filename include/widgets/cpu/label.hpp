#pragma once

#include <gtkmm.h>

namespace widgets {
namespace cpu {
class Label : public Gtk::Label {
public:
  Label();
  void set_load(size_t load);
};

} // namespace cpu
} // namespace widgets

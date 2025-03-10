#include "include/widgets/cpu/label.hpp"

namespace widgets {

namespace cpu {

Label::Label() : Gtk::Label() { set_use_markup(true); }

#define INDICATORS_COUNT 8
static const char *INDICATORS[INDICATORS_COUNT] = {
    "<span color='#FFFFFF'>▁</span>", "<span color='#FFD5D5'>▂</span>",
    "<span color='#FFAAAA'>▃</span>", "<span color='#FF8080'>▄</span>",
    "<span color='#FF5555'>▅</span>", "<span color='#FF2B2B'>▆</span>",
    "<span color='#FF0000'>▇</span>", "<span color='#E60000'>█</span>",
};

void Label::set_load(size_t load) {
  auto indicator_idx =
      (size_t)((double)load / 100.0 * (double)INDICATORS_COUNT);

  if (indicator_idx == INDICATORS_COUNT) {
    indicator_idx -= 1;
  }

  auto markup = INDICATORS[indicator_idx];
  set_label(markup);
}

} // namespace cpu

} // namespace widgets

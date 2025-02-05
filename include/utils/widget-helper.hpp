#pragma once

#include <gtkmm.h>

namespace utils {

template <class T> class WidgetHelper {
public:
  Gdk::Graphene::Point bottom_right_point(Gtk::Widget &window) {
    T *self = static_cast<T *>(this);
    std::optional<Gdk::Graphene::Rect> bounds = self->compute_bounds(window);
    auto x = bounds->get_origin().get_x();
    auto y = bounds->get_origin().get_y();
    auto width = bounds->get_size().get_width();
    auto height = bounds->get_size().get_height();
    return Gdk::Graphene::Point(x + width, y + height);
  }

  Gdk::Graphene::Rect bottom_center_point() const;
};

} // namespace utils

#pragma once

#include "bindings.hpp"

namespace utils {

template <class T> class Subscription {
public:
  void subscribe_to_io_events(void *subscriptions) {
    layer_shell_io::layer_shell_io_subscribe(Subscription::handle_event, this,
                                             subscriptions);
  }

private:
  static void handle_event(const layer_shell_io::Event *event, void *data) {
    ((T *)data)->on_io_event(event);
  }
};

} // namespace utils

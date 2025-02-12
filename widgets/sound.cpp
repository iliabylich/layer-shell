#include "include/widgets/sound.hpp"

namespace widgets {

Sound::Sound(void *ctx) : Gtk::Box(), utils::Subscriber(ctx) {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(5);
  set_css_classes({"widget", "sound", "padded"});
  set_name("Sound");

  image.set_from_icon_name("dialog-question");
  append(image);

  scale.set_orientation(Gtk::Orientation::HORIZONTAL);
  scale.set_adjustment(Gtk::Adjustment::create(0.0, 0.0, 1.0));
  scale.set_css_classes({"sound-slider"});
  append(scale);

  auto ctrl = Gtk::GestureClick::create();
  ctrl->signal_released().connect([this, ctx](int, double, double) {
    auto adj = this->scale.get_adjustment();
    double volume = CLAMP(adj->get_value(), 0.0, 1.0);
    layer_shell_io::layer_shell_io_set_volume(volume, ctx);
  });
  ctrl->set_propagation_phase(Gtk::PropagationPhase::CAPTURE);
  add_controller(ctrl);
}

void Sound::on_io_event(layer_shell_io::Event::Volume_Body data) {
  double volume = data.volume;
  scale.set_value(volume);
  const char *icon_name = NULL;
  if (volume == 0.0) {
    icon_name = "audio-volume-muted-symbolic";
  } else if (volume > 0.01 && volume < 0.34) {
    icon_name = "audio-volume-low-symbolic";
  } else if (volume > 0.34 && volume < 0.67) {
    icon_name = "audio-volume-medium-symbolic";
  } else if (volume > 0.67 && volume < 1.0) {
    icon_name = "audio-volume-high-symbolic";
  } else {
    icon_name = "audio-volume-overamplified-symbolic";
  }
  image.set_from_icon_name(icon_name);
}

} // namespace widgets

#include "include/widgets/sound.hpp"

namespace widgets {

Sound::Sound(io::UiCtx *ui_ctx) : Gtk::Box(), utils::Subscriber(ui_ctx) {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(5);
  set_css_classes({"widget", "sound", "padded"});
  set_name("Sound");

  image.set_from_icon_name("dialog-question");
  append(image);
}

void Sound::on_io_event(io::Event::Volume_Body data) {
  uint32_t volume = data.volume;
  bool muted = data.muted;
  const char *icon_name = NULL;
  if (volume == 0 || muted) {
    icon_name = "audio-volume-muted-symbolic";
  } else if (volume >= 1 && volume < 34) {
    icon_name = "audio-volume-low-symbolic";
  } else if (volume >= 34 && volume < 67) {
    icon_name = "audio-volume-medium-symbolic";
  } else if (volume >= 67 && volume < 95) {
    icon_name = "audio-volume-high-symbolic";
  } else {
    icon_name = "audio-volume-overamplified-symbolic";
  }
  image.set_from_icon_name(icon_name);
}

} // namespace widgets

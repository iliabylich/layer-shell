#include "ui/include/icons.h"
#include "all-icons-embedded.h"
#include <stdint.h>

static GIcon *load_texture(const uint8_t *icon, size_t len) {
  GBytes *bytes = g_bytes_new_static(icon, len);
  return g_bytes_icon_new(bytes);
}

#define X(name)                                                                \
  static GIcon *name##_icon;                                                   \
  GIcon *get_##name##_icon() { return name##_icon; }
#include "x-icons.h"
#undef X

void init_icons(void) {
#define X(name)                                                                \
  name##_icon =                                                                \
      load_texture(___ui_icons_##name##_png, ___ui_icons_##name##_png_len);
#include "x-icons.h"
#undef X
}

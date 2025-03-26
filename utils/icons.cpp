#include "include/utils/icons.hpp"
#include "all-icons-embedded.hpp"

Glib::RefPtr<const Gio::Icon> init_icon(unsigned char *data, size_t size) {
  GBytes *g_bytes = g_bytes_new_static(data, size);
  GIcon *icon_ptr = g_bytes_icon_new(g_bytes);
  auto variant_ptr = g_icon_serialize(icon_ptr);
  auto variant = Glib::wrap(variant_ptr, true);
  return Gio::Icon::deserialize(variant);
}

namespace utils {

void Icons::init() {
#define X(name)                                                                \
  name = init_icon(CONCAT3(___icons_, name, _png),                             \
                   CONCAT3(___icons_, name, _png_len));
#include "include/utils/all-icons.hpp"
#undef X
}

#define X(name) Glib::RefPtr<const Gio::Icon> Icons::name;
#include "include/utils/all-icons.hpp"
#undef X

} // namespace utils

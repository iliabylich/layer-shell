#include "include/utils/strings.hpp"
#include <cstring>

namespace utils {
namespace strings {

char *s(const char *src) {
  char *out = (char *)malloc(strlen(src) + 1);
  strcpy(out, src);
  return out;
}

} // namespace strings
} // namespace utils

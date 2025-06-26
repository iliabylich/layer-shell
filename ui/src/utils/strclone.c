#include "ui/include/utils/strclone.h"
#include <stdlib.h>
#include <string.h>

char *strclone(const char *s) {
  if (s == NULL) {
    return NULL;
  }

  size_t len = strlen(s) + 1;
  char *out = malloc(len);
  if (out == NULL) {
    return NULL;
  }

  memcpy(out, s, len);
  return out;
}

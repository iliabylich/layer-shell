#include "ui/buffer.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static buffer_t buffer_empty() { return (buffer_t){.ptr = NULL, .len = 0}; }

static bool buffer_is_empty(buffer_t buffer) {
  return buffer.ptr == NULL && buffer.len == 0;
}

buffer_t buffer_from_file(const char *path) {
  char *buffer = 0;
  long length;
  FILE *f = fopen(path, "rb");

  if (f) {
    fseek(f, 0, SEEK_END);
    length = ftell(f);
    fseek(f, 0, SEEK_SET);
    buffer = malloc(length + 2);
    fread(buffer, 1, length, f);
    fclose(f);
    buffer[length] = 0;
    return buffer_from_string(buffer, length);
  } else {
    return buffer_empty();
  }
}

buffer_t buffer_from_string(char *ptr, size_t len) {
  return (buffer_t){.ptr = ptr, .len = len};
}

buffer_t buffer_from_const_string(const char *ptr, size_t len) {
  char *out = malloc((len + 1) * sizeof(char));
  memcpy(out, ptr, len);
  out[len] = 0;
  return (buffer_t){.ptr = out, .len = len};
}

buffer_t buffer_merge(buffer_t first, buffer_t second) {
  if (buffer_is_empty(first)) {
    return second;
  }
  if (buffer_is_empty(second)) {
    return first;
  }
  char *out = malloc((first.len + second.len + 1) * sizeof(char));
  memcpy(out, first.ptr, first.len);
  memcpy(out + first.len, second.ptr, second.len);
  out[second.len + first.len] = 0;

  buffer_free(first);
  buffer_free(second);

  return (buffer_t){.ptr = out, .len = first.len + second.len};
}

void buffer_free(buffer_t buffer) { free(buffer.ptr); }

#include "ui/include/buffer.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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
    fprintf(stderr, "Failed to read %s, exiting...\n", path);
    exit(EXIT_FAILURE);
  }
}

buffer_t buffer_from_string(char *ptr, size_t len) {
  return (buffer_t){.ptr = ptr, .len = len, .owned = true};
}

buffer_t buffer_from_const_string(const char *ptr, size_t len) {
  return (buffer_t){.ptr = (char *)ptr, .len = len, .owned = false};
}

buffer_t buffer_merge(buffer_t first, buffer_t second) {
  char *out = calloc(first.len + second.len + 1, sizeof(char));
  memcpy(out, first.ptr, first.len);
  memcpy(out + first.len, second.ptr, second.len);
  out[second.len + first.len] = 0;

  buffer_free(first);
  buffer_free(second);

  return buffer_from_string(out, first.len + second.len);
}

void buffer_free(buffer_t buffer) {
  if (buffer.owned) {
    free(buffer.ptr);
  }
}

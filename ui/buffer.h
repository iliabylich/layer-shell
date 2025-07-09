#pragma once

#include <stddef.h>

typedef struct {
  char *ptr;
  size_t len;
} buffer_t;

buffer_t buffer_from_file(const char *path);
buffer_t buffer_from_string(char *ptr, size_t len);
buffer_t buffer_from_const_string(const char *ptr, size_t len);
buffer_t buffer_merge(buffer_t first, buffer_t second);
void buffer_free(buffer_t buffer);

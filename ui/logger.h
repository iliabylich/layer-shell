#pragma once

#include <stdint.h>

void __log__(uint8_t level, const char *tag, const char *message);

#define LOGGER(tag, level)                                                     \
  static void LOG(const char *message) { __log__(level, tag, message); }

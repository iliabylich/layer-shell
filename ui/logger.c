#include "ui/logger.h"
#include <stdio.h>
#include <string.h>

void __log__(uint8_t level, const char *tag, const char *message) {
  char buffer[100];
  uint8_t indent = level * 4;
  memset(buffer, ' ', indent);
  char *p = buffer + indent;
  *p++ = '[';
  strcpy(p, tag);
  p += strlen(tag);
  *p++ = ']';
  *p++ = ' ';
  strcpy(p, message);
  p += strlen(message);
  *p++ = '\n';
  *p = '\0';
  fputs(buffer, stderr);
}

#pragma once

#include <stddef.h>

struct IO_TrayItem;
typedef struct IO_FFIArray_TrayItem {
  struct IO_TrayItem *ptr;
  size_t len;
} IO_FFIArray_TrayItem;

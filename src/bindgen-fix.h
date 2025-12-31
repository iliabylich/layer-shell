#pragma once

#include <stddef.h>

#define DECLARE_ARRAY(Type)                                                    \
  struct IO_##Type;                                                            \
  typedef struct IO_CArray_##Type {                                            \
    struct IO_##Type *ptr;                                                     \
    size_t len;                                                                \
  } IO_CArray_##Type;

DECLARE_ARRAY(TrayItem)

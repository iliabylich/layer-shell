#define _CONCAT(a, b) a##b
#define CONCAT(a, b) _CONCAT(a, b)
#define CONCAT3(a, b, c) CONCAT(a, CONCAT(b, c))

#ifndef X
#define X(name)
#endif

#include "icons/x-icons.hpp"

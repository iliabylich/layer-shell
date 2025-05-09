#pragma once

#include <gio/gio.h>

void init_icons(void);

#define X(name) GIcon *get_##name##_icon();
#include "x-icons.h"
#undef X

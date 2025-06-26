#pragma once

#define int_to_tray_action_name_prefixed(i, buf)                               \
  char buf[100];                                                               \
  sprintf(buf, "tray.%d", i);

#define int_to_tray_action_name_no_prefix(i, buf)                              \
  char buf[100];                                                               \
  sprintf(buf, "%d", i);

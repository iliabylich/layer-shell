#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(NetworkPopover, network_popover, NETWORK_POPOVER, WIDGET,
                     GtkWidget)

#define NETWORK_POPOVER(obj)                                                   \
  G_TYPE_CHECK_INSTANCE_CAST(obj, network_popover_get_type(), NetworkPopover)

GtkWidget *network_popover_new(void);
void network_popover_open(NetworkPopover *popover);

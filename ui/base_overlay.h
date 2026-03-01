#pragma once

#include <gtk/gtk.h>

G_DECLARE_DERIVABLE_TYPE(BaseOverlay, base_overlay, BASE_OVERLAY, WINDOW,
                         GtkWindow)

#define BASE_OVERLAY_TYPE base_overlay_get_type()
#define BASE_OVERLAY(obj)                                                      \
  G_TYPE_CHECK_INSTANCE_CAST(obj, base_overlay_get_type(), BaseOverlay)

struct _BaseOverlayClass {
  GtkWindowClass parent_class;
};

void base_overlay_vte(BaseOverlay *self, char **command);

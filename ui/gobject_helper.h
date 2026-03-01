#pragma once

#include <glib-object.h>

gboolean gobject_get_nested_bool(GObject *object, const char *child_property,
                                 const char *nested_property);
void gobject_set_nested_impl(GObject *object, const char *child_property,
                             const char *nested_property, ...);
#define gobject_set_nested(object, child_property, nested_property, ...)       \
  gobject_set_nested_impl(object, child_property, nested_property,             \
                          __VA_ARGS__, NULL)
void gobject_toggle_nested(GObject *object, const char *child_property,
                           const char *nested_property);

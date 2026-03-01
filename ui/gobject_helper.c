#include "ui/gobject_helper.h"
#include <stdarg.h>

static GObject *get_required_child(GObject *object,
                                   const char *child_property) {
  GParamSpec *child_pspec =
      g_object_class_find_property(G_OBJECT_GET_CLASS(object), child_property);
  if (!child_pspec) {
    g_error("Missing child property '%s' on type '%s'", child_property,
            G_OBJECT_TYPE_NAME(object));
  }
  if (!G_IS_PARAM_SPEC_OBJECT(child_pspec)) {
    g_error("Property '%s' on type '%s' is not an object property",
            child_property, G_OBJECT_TYPE_NAME(object));
  }

  GObject *child = NULL;
  g_object_get(object, child_property, &child, NULL);
  if (!child) {
    g_error("Child property '%s' on type '%s' is NULL", child_property,
            G_OBJECT_TYPE_NAME(object));
  }
  return child;
}

static GParamSpec *get_required_nested_pspec(GObject *child,
                                             const char *child_property,
                                             const char *nested_property) {
  GParamSpec *nested_pspec =
      g_object_class_find_property(G_OBJECT_GET_CLASS(child), nested_property);
  if (!nested_pspec) {
    g_error("Missing nested property '%s.%s' (child type '%s')", child_property,
            nested_property, G_OBJECT_TYPE_NAME(child));
  }
  return nested_pspec;
}

gboolean gobject_get_nested_bool(GObject *object, const char *child_property,
                                 const char *nested_property) {
  GObject *child = get_required_child(object, child_property);
  GParamSpec *nested_pspec =
      get_required_nested_pspec(child, child_property, nested_property);
  if (!G_IS_PARAM_SPEC_BOOLEAN(nested_pspec)) {
    g_error("Nested property '%s.%s' is not boolean (child type '%s')",
            child_property, nested_property, G_OBJECT_TYPE_NAME(child));
  }
  gboolean value = false;
  g_object_get(child, nested_property, &value, NULL);
  g_object_unref(child);
  return value;
}

void gobject_set_nested_impl(GObject *object, const char *child_property,
                             const char *nested_property, ...) {
  GObject *child = get_required_child(object, child_property);
  get_required_nested_pspec(child, child_property, nested_property);
  va_list args;
  va_start(args, nested_property);
  g_object_set_valist(child, nested_property, args);
  va_end(args);
  g_object_unref(child);
}

void gobject_toggle_nested(GObject *object, const char *child_property,
                           const char *nested_property) {
  gobject_set_nested(
      object, child_property, nested_property,
      !gobject_get_nested_bool(object, child_property, nested_property));
}

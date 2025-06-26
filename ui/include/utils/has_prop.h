#pragma once

#define stringify(s) #s

#define WIDGET_HAS_PROP(name, Type)                                            \
  static void __free__##name(void *) {}                                        \
                                                                               \
  static void set_##name(GtkWidget *self, Type value) {                        \
    g_object_set_data_full(G_OBJECT(self), stringify(name), (void *)value,     \
                           __free__##name);                                    \
  }                                                                            \
                                                                               \
  static Type get_##name(GtkWidget *self) {                                    \
    return (Type)(size_t)g_object_get_data(G_OBJECT(self), stringify(name));   \
  }

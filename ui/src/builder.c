#include "ui/include/builder.h"
#include "htop.xml.xxd"
#include "launcher.xml.xxd"
#include "ping.xml.xxd"
#include "session.xml.xxd"
#include "top_bar.xml.xxd"
#include "weather.xml.xxd"

#define IMPL_BUILDER(name)                                                     \
  GtkBuilder *name##_builder = NULL;                                           \
  static void init_##name##_builder() {                                        \
    name##_builder =                                                           \
        gtk_builder_new_from_string((const char *)name##_xml, name##_xml_len); \
  }                                                                            \
  GtkWidget *name##_get_widget(const char *id) {                               \
    return GTK_WIDGET(gtk_builder_get_object(name##_builder, id));             \
  }

IMPL_BUILDER(htop)
IMPL_BUILDER(launcher)
IMPL_BUILDER(ping)
IMPL_BUILDER(session)
IMPL_BUILDER(top_bar)
IMPL_BUILDER(weather)

#undef IMPL_BUILDER

void init_builders() {
  init_htop_builder();
  init_launcher_builder();
  init_ping_builder();
  init_session_builder();
  init_top_bar_builder();
  init_weather_builder();
}
